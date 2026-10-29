mod commands;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod services;
pub mod state;

pub fn run() {
    use services::engine::Engine;
    use std::sync::Arc;
    use tauri::{Emitter, Manager};
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            services::window::show(app)
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_snapshot,
            commands::timer_action,
            commands::task_action,
            commands::settings_update,
            commands::window_set_mode,
            commands::window_close,
            commands::audio_preview,
            commands::audio_import_custom
        ])
        .setup(|app| {
            let storage = infrastructure::storage::Storage::new(app.path().app_data_dir()?)?;
            match Engine::new(app.handle().clone(), storage) {
                Ok(engine) => {
                    services::window::apply(app.handle(), &engine.snapshot()?, true)?;
                    app.manage(engine.clone());
                    infrastructure::tray::install(app.handle())?;
                    Engine::spawn(engine);
                    if let Some(window) = app.get_webview_window("main") {
                        window.show()?;
                    }
                }
                Err(error) => {
                    use tauri_plugin_dialog::DialogExt;
                    app.dialog()
                        .message(error.to_string())
                        .title("RVNFlow · data recovery needed")
                        .blocking_show();
                    return Err(Box::new(error));
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            let app = window.app_handle();
            let Some(engine) = app.try_state::<Arc<Engine>>() else {
                return;
            };
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if let Ok(data) = engine.snapshot()
                        && data.settings.close_to_tray
                    {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    if let Err(error) = engine.update("window:changed", |_| Ok(())) {
                        api.prevent_close();
                        let _ = app.emit("app:error", error.to_string());
                    }
                }
                tauri::WindowEvent::Moved(position) => {
                    // Keep moves in memory; save on focus loss, close, or next mutation.
                    if let Ok(mut data) = engine.data.lock()
                        && data.settings.remember_position
                    {
                        data.window.position = Some((position.x, position.y));
                    }
                }
                tauri::WindowEvent::Focused(false) => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = services::window::clamp(&w);
                    }
                    let _ = engine.update("window:changed", |_| Ok(()));
                }
                tauri::WindowEvent::ScaleFactorChanged { .. } => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = services::window::clamp(&w);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("RVNFlow could not start: {error}");
    }
}
