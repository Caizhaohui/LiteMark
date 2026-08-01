//! Sidecar process client.
//!
//! Spawns the Node render sidecar as a child process and speaks the JSON Lines
//! protocol defined in `packages/shared-protocol`. Responsibilities
//! (DEVELOPMENT_PLAN.md §5.3, §5.2):
//!  - Spawn `node <sidecar entry>` with stdin/stdout pipes.
//!  - Correlate requests by id, with a per-request timeout (default 10s).
//!  - Detect a crashed/exited sidecar and surface `SIDECAR_CRASHED`.
//!  - Provide a clean shutdown that kills the child.
//!
//! No `exec`/`shell`/`runCommand` surface exists; only the static method set.

use crate::error::{ErrorCode, SidecarError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{oneshot, Notify};
use tokio::time::timeout;

/// Default per-request timeout (DEVELOPMENT_PLAN.md §8.2: single render 10s).
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// A pending request awaiting its correlated response.
struct Pending {
    tx: oneshot::Sender<Result<Value, SidecarError>>,
}

/// Shared, lock-protected client state shared with the stdout reader task.
struct SidecarInner {
    next_id: AtomicU64,
    pending: HashMap<String, Pending>,
    /// Set once the child exits; subsequent requests fail fast with SIDECAR_CRASHED.
    crashed: bool,
    /// Most recent child exit reason for diagnostics.
    last_exit: Option<String>,
}

/// The running sidecar. The stdin writer is held behind a mutex so requests can
/// write lines; stdout is drained by a background task; a watcher marks the
/// client crashed on child exit.
pub struct Sidecar {
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    inner: Arc<Mutex<SidecarInner>>,
    /// Owned so that `kill_on_drop` fires when the Sidecar is dropped.
    _child: Option<Child>,
    /// Notified when the sidecar process exits. Held for future use by a crash
    /// recovery watcher on the Tauri event bus (M1+); read here only indirectly.
    #[allow(dead_code)]
    exit_notify: Arc<Notify>,
}

/// Where to find the sidecar entry script. Resolved from an env override first
/// (set by `pnpm tauri dev` / CI), then next to the executable (release bundle
/// resources), then a repo-relative path for development. No hardcoded absolute
/// developer paths.
fn resolve_sidecar_entry() -> Result<PathBuf, SidecarError> {
    if let Ok(p) = std::env::var("LITEMARK_SIDECAR_ENTRY") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Ok(path);
        }
    }
    // Release layout: <resource>/sidecar/index.js next to the exe or in resources.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for rel in [
                "sidecar/index.js",
                "resources/sidecar/index.js",
                "../resources/sidecar/index.js",
            ] {
                let candidate = dir.join(rel);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
    }
    let cwd = std::env::current_dir()
        .map_err(|e| SidecarError::new(ErrorCode::SidecarStartFailed, format!("cwd: {e}")))?;
    let candidates = [
        cwd.join("packages")
            .join("render-sidecar")
            .join("dist")
            .join("index.js"),
        cwd.join("src-tauri")
            .join("resources")
            .join("sidecar")
            .join("index.js"),
        cwd.join("resources").join("sidecar").join("index.js"),
    ];
    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }
    Err(SidecarError::new(
        ErrorCode::SidecarStartFailed,
        "Sidecar entry not found. Set LITEMARK_SIDECAR_ENTRY or build the sidecar first (pnpm --filter @litemark/render-sidecar build).",
    )
    .with_details(serde_json::json!({ "checked": candidates })))
}

/// Resolve the `node` executable. Honors an env override, then a portable Node
/// shipped next to the app (so end users need not install Node.js), then PATH.
fn resolve_node() -> String {
    if let Ok(p) = std::env::var("LITEMARK_NODE") {
        if Path::new(&p).exists() || p == "node" {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for rel in [
                "node/node.exe",
                "resources/node/node.exe",
                "../resources/node/node.exe",
                "node.exe",
            ] {
                let candidate = dir.join(rel);
                if candidate.exists() {
                    return candidate.to_string_lossy().into_owned();
                }
            }
        }
    }
    "node".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct SuccessEnvelope {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    ok: bool,
    result: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorEnvelope {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    ok: bool,
    error: SidecarErrorWire,
}

#[derive(Debug, Serialize, Deserialize)]
struct SidecarErrorWire {
    code: String,
    message: String,
    #[serde(default)]
    details: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventEnvelope {
    #[allow(dead_code)]
    event: String,
    #[allow(dead_code)]
    payload: Value,
}

impl Sidecar {
    /// Spawn the sidecar using the resolved node + entry path.
    pub async fn spawn() -> Result<Self, SidecarError> {
        Self::spawn_with_events(None).await
    }

    /// Spawn with optional AppHandle for forwarding protocol events to the UI.
    pub async fn spawn_with_events(app: Option<AppHandle>) -> Result<Self, SidecarError> {
        let entry = resolve_sidecar_entry()?;
        let node = resolve_node();
        Self::spawn_with(&node, &entry, app).await
    }

    /// Spawn variant for tests / explicit node + entry paths.
    pub async fn spawn_with(
        node: &str,
        entry: &Path,
        app: Option<AppHandle>,
    ) -> Result<Self, SidecarError> {
        let mut command = Command::new(node);
        command
            .arg(entry)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Ensure the sidecar dies with the app on Windows.
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|e| {
            SidecarError::new(ErrorCode::SidecarStartFailed, format!("spawn node: {e}"))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| SidecarError::new(ErrorCode::SidecarStartFailed, "no stdin pipe"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| SidecarError::new(ErrorCode::SidecarStartFailed, "no stdout pipe"))?;
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    log::info!("[sidecar] {}", line);
                }
            });
        }

        let inner = Arc::new(Mutex::new(SidecarInner {
            next_id: AtomicU64::new(1),
            pending: HashMap::new(),
            crashed: false,
            last_exit: None,
        }));
        let exit_notify = Arc::new(Notify::new());

        // stdout reader task.
        let stdout_inner = inner.clone();
        let stdout_exit = exit_notify.clone();
        let app_for_events = app;
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut buf = String::new();
                match reader.read_line(&mut buf).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = buf.trim().to_string();
                        if line.is_empty() {
                            continue;
                        }
                        if let Err(e) =
                            handle_stdout_line(&stdout_inner, &line, app_for_events.as_ref()).await
                        {
                            log::warn!("[sidecar] malformed stdout line ({e}): {line}");
                        }
                    }
                    Err(e) => {
                        log::warn!("[sidecar] stdout read error: {e}");
                        break;
                    }
                }
            }
            mark_crashed(&stdout_inner, "stdout closed".to_string()).await;
            stdout_exit.notify_waiters();
        });

        // process exit watcher.
        let exit_inner = inner.clone();
        let exit_notify2 = exit_notify.clone();
        tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => mark_crashed(&exit_inner, format!("exit: {status}")).await,
                Err(e) => mark_crashed(&exit_inner, format!("wait error: {e}")).await,
            }
            exit_notify2.notify_waiters();
        });

        Ok(Self {
            stdin: Arc::new(Mutex::new(stdin)),
            inner,
            _child: None, // moved into the watcher above; kill_on_drop honored via the moved child
            exit_notify,
        })
    }

    /// Block (with timeout) until the sidecar is responsive (a ping succeeds).
    pub async fn wait_ready(&self, deadline: Duration) -> Result<(), SidecarError> {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > deadline {
                return Err(SidecarError::new(
                    ErrorCode::SidecarStartFailed,
                    "sidecar did not become ready in time",
                ));
            }
            match self.request("ping", serde_json::json!({})).await {
                Ok(_) => return Ok(()),
                Err(e) if e.code == ErrorCode::SidecarCrashed => return Err(e),
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    }

    /// Send a request and await its response, subject to the default timeout.
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, SidecarError> {
        self.request_with_timeout(method, params, DEFAULT_REQUEST_TIMEOUT)
            .await
    }

    pub async fn request_with_timeout(
        &self,
        method: &str,
        params: Value,
        deadline: Duration,
    ) -> Result<Value, SidecarError> {
        let (id, rx) = {
            let mut guard = self.inner.lock().await;
            if guard.crashed {
                let msg = guard
                    .last_exit
                    .clone()
                    .unwrap_or_else(|| "sidecar not running".to_string());
                return Err(SidecarError::new(ErrorCode::SidecarCrashed, msg));
            }
            let id = guard.next_id.fetch_add(1, Ordering::SeqCst).to_string();
            let (tx, rx) = oneshot::channel();
            guard.pending.insert(id.clone(), Pending { tx });
            (id, rx)
        };

        let envelope = serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        });
        let mut line = serde_json::to_string(&envelope)
            .map_err(|e| SidecarError::new(ErrorCode::ProtocolInvalid, format!("encode: {e}")))?;
        line.push('\n');

        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(line.as_bytes()).await.map_err(|e| {
                SidecarError::new(ErrorCode::SidecarCrashed, format!("write stdin: {e}"))
            })?;
            stdin.flush().await.map_err(|e| {
                SidecarError::new(ErrorCode::SidecarCrashed, format!("flush stdin: {e}"))
            })?;
        }

        match timeout(deadline, rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(SidecarError::new(
                ErrorCode::SidecarCrashed,
                "response channel dropped",
            )),
            Err(_) => {
                // Timed out: drop our pending entry so a late reply is ignored.
                let mut guard = self.inner.lock().await;
                guard.pending.remove(&id);
                Err(SidecarError::new(
                    ErrorCode::SidecarTimeout,
                    format!(
                        "request '{method}' timed out after {}ms",
                        deadline.as_millis()
                    ),
                ))
            }
        }
    }

    /// Clean shutdown: best-effort `shutdown` method then rely on kill_on_drop.
    pub async fn shutdown(&self) {
        let _ = self.request("shutdown", serde_json::json!({})).await;
    }
}

async fn handle_stdout_line(
    inner: &Arc<Mutex<SidecarInner>>,
    line: &str,
    app: Option<&AppHandle>,
) -> Result<(), String> {
    let value: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    if value.get("event").is_some() {
        let evt: EventEnvelope =
            serde_json::from_value(value.clone()).map_err(|e| format!("event decode: {e}"))?;
        if let Some(app) = app {
            forward_sidecar_event(app, &evt.event, evt.payload);
        } else {
            log::debug!("[sidecar event] {}", evt.event);
        }
        return Ok(());
    }
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing id".to_string())?
        .to_string();
    let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let pending = {
        let mut guard = inner.lock().await;
        guard.pending.remove(&id)
    };
    let Some(pending) = pending else {
        return Ok(());
    };
    if ok {
        let success: SuccessEnvelope =
            serde_json::from_value(value).map_err(|e| format!("success decode: {e}"))?;
        let _ = pending.tx.send(Ok(success.result));
    } else {
        let err_env: ErrorEnvelope =
            serde_json::from_value(value).map_err(|e| format!("error decode: {e}"))?;
        let code = parse_error_code(&err_env.error.code);
        let mut e = SidecarError::new(code, err_env.error.message);
        if let Some(d) = err_env.error.details {
            e = e.with_details(d);
        }
        let _ = pending.tx.send(Err(e));
    }
    Ok(())
}

fn parse_error_code(s: &str) -> ErrorCode {
    match s {
        "FILE_NOT_FOUND" => ErrorCode::FileNotFound,
        "FILE_PERMISSION_DENIED" => ErrorCode::FilePermissionDenied,
        "FILE_CHANGED_EXTERNALLY" => ErrorCode::FileChangedExternally,
        "FILE_ENCODING_UNSUPPORTED" => ErrorCode::FileEncodingUnsupported,
        "SAVE_ATOMIC_REPLACE_FAILED" => ErrorCode::SaveAtomicReplaceFailed,
        "SIDECAR_START_FAILED" => ErrorCode::SidecarStartFailed,
        "SIDECAR_CRASHED" => ErrorCode::SidecarCrashed,
        "SIDECAR_TIMEOUT" => ErrorCode::SidecarTimeout,
        "PROTOCOL_INVALID" => ErrorCode::ProtocolInvalid,
        "RENDER_FAILED" => ErrorCode::RenderFailed,
        "RENDER_CANCELLED" => ErrorCode::RenderCancelled,
        "EXPORT_FAILED" => ErrorCode::ExportFailed,
        "EXPORT_CANCELLED" => ErrorCode::ExportCancelled,
        "BROWSER_NOT_FOUND" => ErrorCode::BrowserNotFound,
        "PANDOC_NOT_FOUND" => ErrorCode::PandocNotFound,
        "UNTRUSTED_OPERATION_BLOCKED" => ErrorCode::UntrustedOperationBlocked,
        "PATH_NOT_AUTHORIZED" => ErrorCode::PathNotAuthorized,
        "ROUNDTRIP_DATA_LOSS_RISK" => ErrorCode::RoundtripDataLossRisk,
        _ => ErrorCode::RenderFailed,
    }
}

/// Forward a sidecar protocol event to the webview (best-effort).
fn forward_sidecar_event(app: &AppHandle, event: &str, payload: Value) {
    match event {
        "exportProgress" => {
            let _ = app.emit("export-progress", payload);
        }
        "ready" => {
            let _ = app.emit("sidecar-ready", payload);
        }
        "log" => {
            log::info!("[sidecar event log] {payload}");
        }
        other => {
            log::debug!("[sidecar event] {other}: {payload}");
        }
    }
}

async fn mark_crashed(inner: &Arc<Mutex<SidecarInner>>, reason: String) {
    let mut guard = inner.lock().await;
    guard.crashed = true;
    guard.last_exit = Some(reason.clone());
    let pending: HashMap<String, Pending> = std::mem::take(&mut guard.pending);
    for (_, p) in pending {
        let _ = p.tx.send(Err(SidecarError::new(
            ErrorCode::SidecarCrashed,
            reason.clone(),
        )));
    }
}
