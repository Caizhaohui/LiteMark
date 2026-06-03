use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::assets;
use crate::config::RuntimeConfig;
use crate::markdown;
use crate::watcher;

pub struct AppState {
    pub file_path: PathBuf,
    pub content: RwLock<String>,
    pub config: RuntimeConfig,
    pub tx: broadcast::Sender<String>,
}

pub async fn run_server(file_path: PathBuf, config: RuntimeConfig) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(&file_path)?;
    let (tx, _) = broadcast::channel::<String>(16);

    let state = Arc::new(AppState {
        file_path: file_path.clone(),
        content: RwLock::new(content),
        config: config.clone(),
        tx: tx.clone(),
    });

    let watch_state = state.clone();
    let watch_file = file_path.clone();
    tokio::spawn(async move {
        if let Err(e) = watcher::watch_file(watch_file, watch_state).await {
            eprintln!("File watcher error: {}", e);
        }
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/assets/{*path}", get(asset_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;

    println!("  LiteMark Preview Server");
    println!("  ─────────────────────────────");
    println!("  File: {}", file_path.display());
    println!("  URL:  http://localhost:{}", actual_addr.port());
    println!();
    println!("  Press Ctrl+C to stop.");

    if config.open_browser {
        let url = format!("http://localhost:{}", actual_addr.port());
        let _ = open::that(&url);
    }

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let content = state.content.read().await;
    let html = markdown::render_for_preview(&content, &state.config, &state.file_path);
    Html(html)
}

async fn asset_handler(Path(path): Path<String>) -> impl IntoResponse {
    if let Some(data) = assets::get_asset_bytes(&path) {
        let mime = assets::mime_type(&path);
        (
            [(axum::http::header::CONTENT_TYPE, mime)],
            data,
        )
            .into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let content = state.content.read().await.clone();
    let parsed = markdown::parse_with_render_config(&content, &state.config.file_config.render);
    let init_msg = serde_json::json!({
        "type": "init",
        "html": parsed.html,
        "title": parsed.title,
        "toc": parsed.toc_html,
    });
    let _ = sender
        .send(Message::Text(init_msg.to_string()))
        .await;

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                    handle_client_message(&cmd, &state_clone).await;
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

async fn handle_client_message(cmd: &serde_json::Value, state: &Arc<AppState>) {
    if let Some(cmd_type) = cmd.get("type").and_then(|v| v.as_str()) {
        match cmd_type {
            "taskToggle" => {
                if let Some(line) = cmd.get("line").and_then(|v| v.as_u64()) {
                    let mut content = state.content.write().await;
                    if let Some(new_content) = toggle_task_checkbox(&content, line as usize) {
                        if let Err(e) = std::fs::write(&state.file_path, &new_content) {
                            eprintln!("Failed to write file: {}", e);
                        } else {
                            *content = new_content.clone();
                            let parsed = markdown::parse_with_render_config(
                                &new_content,
                                &state.config.file_config.render,
                            );
                            let msg = serde_json::json!({
                                "type": "update",
                                "html": parsed.html,
                                "title": parsed.title,
                                "toc": parsed.toc_html,
                            });
                            let _ = state.tx.send(msg.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn toggle_task_checkbox(content: &str, line: usize) -> Option<String> {
    let had_trailing_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if line == 0 || line > lines.len() {
        return None;
    }

    let idx = line - 1;
    let line_text = &lines[idx];

    let new_line = if line_text.contains("[ ]") {
        line_text.replacen("[ ]", "[x]", 1)
    } else if line_text.contains("[x]") || line_text.contains("[X]") {
        line_text
            .replacen("[x]", "[ ]", 1)
            .replacen("[X]", "[ ]", 1)
    } else {
        return None;
    };

    lines[idx] = new_line;

    let mut result = lines.join("\n");
    if had_trailing_newline {
        result.push('\n');
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_unchecked_to_checked() {
        let content = "- [ ] task one\n- [ ] task two\n";
        let result = toggle_task_checkbox(content, 1).unwrap();
        assert!(result.contains("[x]"));
        assert!(result.contains("[ ]"));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_toggle_checked_to_unchecked() {
        let content = "- [x] done task\n- [ ] pending\n";
        let result = toggle_task_checkbox(content, 1).unwrap();
        assert!(result.contains("[ ]"));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_toggle_preserves_no_trailing_newline() {
        let content = "- [ ] task one\n- [ ] task two";
        let result = toggle_task_checkbox(content, 2).unwrap();
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn test_toggle_invalid_line() {
        let content = "- [ ] task\n";
        assert!(toggle_task_checkbox(content, 0).is_none());
        assert!(toggle_task_checkbox(content, 99).is_none());
    }

    #[test]
    fn test_toggle_no_checkbox() {
        let content = "just a line\n";
        assert!(toggle_task_checkbox(content, 1).is_none());
    }
}
