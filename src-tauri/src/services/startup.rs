use crate::{
    error::{Result, invalid},
    infrastructure::{diagnostics, storage::Storage},
    services::{engine::Engine, window},
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{Emitter, Manager};
#[derive(Default)]
pub struct Startup {
    pub engine: Mutex<Option<Arc<Engine>>>,
    error: Mutex<Option<String>>,
    loading: AtomicBool,
}
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    pub ready: bool,
    pub loading: bool,
    pub error: Option<String>,
}
impl Startup {
    pub fn ready(&self) -> Result<Arc<Engine>> {
        self.engine
            .lock()
            .map_err(|_| invalid("Startup is unavailable."))?
            .clone()
            .ok_or_else(|| invalid("RVNFlow is loading. Please wait or retry recovery."))
    }
    pub fn status(&self) -> StartupStatus {
        StartupStatus {
            ready: self.ready().is_ok(),
            loading: self.loading.load(Ordering::Relaxed),
            error: self.error.lock().ok().and_then(|e| e.clone()),
        }
    }
}
pub fn launch(app: tauri::AppHandle, directory: PathBuf) {
    let state = app.state::<Startup>();
    if state.ready().is_ok() || state.loading.swap(true, Ordering::Relaxed) {
        return;
    }
    tauri::async_runtime::spawn_blocking(move || {
        diagnostics::init(directory.clone());
        let result = Storage::new(directory).and_then(|storage| Engine::new(app.clone(), storage));
        let state = app.state::<Startup>();
        match result {
            Ok(engine) => {
                let result = engine
                    .update("app:initializing", |_| Ok(()))
                    .and_then(|data| window::apply(&app, &data, true));
                if result.is_err() {
                    if let Ok(mut slot) = state.engine.lock() {
                        *slot = None;
                    }
                    if let Ok(mut error) = state.error.lock() {
                        *error=Some("Unable to restore or save your widget. Your task files are preserved. Check available disk space and retry recovery.".into());
                    }
                    diagnostics::record("startup", "restore-failed");
                    state.loading.store(false, Ordering::Relaxed);
                    let _ = app.emit("startup:changed", state.status());
                    return;
                }
                if let Ok(mut slot) = state.engine.lock() {
                    *slot = Some(engine.clone());
                }
                Engine::spawn(engine.clone());
                #[cfg(debug_assertions)]
                crate::infrastructure::smoke::start(engine.clone());
                if let Ok(data) = engine.snapshot() {
                    let _ = app.emit("app:ready", data);
                }
                if let Ok(mut error) = state.error.lock() {
                    *error = None;
                }
                diagnostics::record("startup", "ready");
            }
            Err(error) => {
                if let Ok(mut message) = state.error.lock() {
                    *message = Some(error.to_string());
                }
                diagnostics::record("startup", "recovery-required");
            }
        }
        state.loading.store(false, Ordering::Relaxed);
        let _ = app.emit("startup:changed", state.status());
    });
}
