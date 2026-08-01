//! SidecarManager: Tauri-managed singleton holding the live Sidecar.
//!
//! Stored as Tauri state so commands can access one shared sidecar. Lazily
//! spawned on first use; restarted automatically after a crash on the next
//! request (the M0 acceptance criterion: "sidecar 崩溃后可重启").
//!
//! M3: supports long-timeout export requests and forwards sidecar events
//! (`exportProgress`) to the webview as Tauri `export-progress` events.

use crate::error::{ErrorCode, SidecarError};
use crate::sidecar::client::{Sidecar, DEFAULT_REQUEST_TIMEOUT};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tauri::async_runtime::Mutex;
use tauri::AppHandle;

#[derive(Clone)]
pub struct SidecarManager {
    sidecar: Arc<Mutex<Option<Sidecar>>>,
    app: Arc<Mutex<Option<AppHandle>>>,
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self {
            sidecar: Arc::new(Mutex::new(None)),
            app: Arc::new(Mutex::new(None)),
        }
    }
}

impl SidecarManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach the AppHandle so sidecar stdout events can be re-emitted to the UI.
    pub async fn set_app_handle(&self, app: AppHandle) {
        *self.app.lock().await = Some(app);
    }

    /// Lazily spawn the sidecar if not already running, then issue a request.
    /// On a crash mid-request, retry once with a freshly-spawned sidecar.
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
        match self
            .request_once(method, params.clone(), deadline)
            .await
        {
            Ok(v) => Ok(v),
            Err(e) if e.code == ErrorCode::SidecarCrashed => {
                self.restart().await?;
                self.request_once(method, params, deadline).await
            }
            Err(e) => Err(e),
        }
    }

    async fn request_once(
        &self,
        method: &str,
        params: Value,
        deadline: Duration,
    ) -> Result<Value, SidecarError> {
        let mut guard = self.sidecar.lock().await;
        if guard.is_none() {
            let app = self.app.lock().await.clone();
            let sidecar = Sidecar::spawn_with_events(app).await?;
            *guard = Some(sidecar);
        }
        let sidecar = guard.as_ref().expect("sidecar present");
        sidecar
            .request_with_timeout(method, params, deadline)
            .await
    }

    /// Drop the current sidecar so the next request respawns it.
    pub async fn restart(&self) -> Result<(), SidecarError> {
        let mut guard = self.sidecar.lock().await;
        if let Some(s) = guard.take() {
            s.shutdown().await;
        }
        Ok(())
    }
}
