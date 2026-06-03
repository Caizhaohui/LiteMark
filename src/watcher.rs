use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;

use crate::markdown;
use crate::server::AppState;

/// Watch a file for changes and push updates to WebSocket clients
pub async fn watch_file(file_path: PathBuf, state: Arc<AppState>, debounce_ms: u64) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<()>(16);

    // Set up the file watcher
    let watch_path = file_path.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) => {
                        let _ = tx.try_send(());
                    }
                    _ => {}
                }
            }
        },
        Config::default(),
    )?;

    watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

    // Debounce: wait for a quiet period before re-rendering (from config)
    let debounce = Duration::from_millis(debounce_ms);

    loop {
        // Wait for a change notification
        if rx.recv().await.is_none() {
            break;
        }

        // Drain any additional events that arrive during the debounce window
        let mut last_event = time::Instant::now();
        loop {
            match time::timeout(debounce, rx.recv()).await {
                Ok(Some(_)) => {
                    last_event = time::Instant::now();
                }
                _ => break,
            }
        }

        // Re-read the file
        match std::fs::read_to_string(&watch_path) {
            Ok(new_content) => {
                // Update shared state
                {
                    let mut content = state.content.write().await;
                    *content = new_content.clone();
                }

                // Parse and broadcast
                let parsed = markdown::parse(&new_content);
                let msg = serde_json::json!({
                    "type": "update",
                    "html": parsed.html,
                    "title": parsed.title,
                    "toc": parsed.toc_html,
                });
                let _ = state.tx.send(msg.to_string());
            }
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
            }
        }
    }

    Ok(())
}
