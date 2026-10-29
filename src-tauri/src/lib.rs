mod commands;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod services;
pub mod state;

pub fn run() {
    use services::{
        startup::{self, Startup},
        window,
    };
    use tauri::Manager;
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let app = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                infrastructure::diagnostics::record("instance", "activate-existing");
                window::show(&app);
            });
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Startup::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_snapshot,
            commands::timer_action,
            commands::task_action,
            commands::settings_update,
            commands::window_set_mode,
            commands::window_close,
            commands::window_minimize,
            commands::window_hide,
            commands::app_quit,
            commands::get_startup_status,
            commands::startup_retry,
            commands::frontend_ready,
            commands::audio_preview,
            commands::audio_import_custom
        ])
        .setup(|app| {
            infrastructure::tray::install(app.handle())?;
            if let Some(window) = app.get_webview_window("main") {
                window.show()?;
            }
            let directory = app.path().app_data_dir()?;
            #[cfg(debug_assertions)]
            let directory = if std::env::var_os("RVNFLOW_NATIVE_SMOKE").is_some() {
                std::env::var_os("RVNFLOW_TEST_DATA_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or(directory)
            } else {
                directory
            };
            startup::launch(app.handle().clone(), directory);
            Ok(())
        })
        .on_window_event(|window, event| {
            let app = window.app_handle();
            let Some(state) = app.try_state::<Startup>() else {
                return;
            };
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let Ok(engine) = state.ready() else {
                        app.exit(0);
                        return;
                    };
                    tauri::async_runtime::spawn_blocking(move || {
                        let result = engine
                            .persist_position()
                            .and_then(|_| engine.update("window:changed", |_| Ok(())));
                        match result {
                            Ok(data) if data.settings.close_to_tray => {
                                let _ = services::window::hide(&engine.app);
                            }
                            Ok(_) => engine.app.exit(0),
                            Err(_) => engine.report(
                                "Unable to save before closing. Check your disk and try again.",
                                "close-save",
                            ),
                        }
                    });
                }
                tauri::WindowEvent::Moved(position) => {
                    if let Ok(engine) = state.ready()
                        && let Ok(mut pending) = engine.position.lock()
                    {
                        *pending = Some((position.x, position.y));
                    }
                }
                tauri::WindowEvent::Focused(false) => {
                    if let Ok(engine) = state.ready() {
                        tauri::async_runtime::spawn_blocking(move || {
                            let _ = engine.persist_position();
                        });
                    }
                }
                tauri::WindowEvent::ScaleFactorChanged { .. } => {
                    let app = app.clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = services::window::clamp(&window);
                        }
                    });
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("RVNFlow could not start: {error}");
    }
}
