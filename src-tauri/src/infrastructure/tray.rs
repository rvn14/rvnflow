use crate::{
    domain::timer::Status,
    infrastructure::clock::{now_ms, wall_ms},
    services::{engine::Engine, window},
};
use std::sync::Arc;
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
pub fn install(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show / hide RVNFlow", true, None::<&str>)?;
    let timer = MenuItem::with_id(app, "timer", "Start / pause session", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit RVNFlow", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &timer, &quit])?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("RVNFlow · A little space to focus")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let engine = app.state::<Arc<Engine>>();
            match event.id.as_ref() {
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            window::show(app);
                        }
                    }
                }
                "timer" => {
                    if let Err(error) = engine.update("timer:changed", |d| {
                        if d.timer.status == Status::Running {
                            d.timer.pause(now_ms());
                        } else {
                            d.timer.start(now_ms(), wall_ms(), &d.settings);
                        }
                        Ok(())
                    }) {
                        use tauri::Emitter;
                        let _ = app.emit("app:error", error.to_string());
                    }
                }
                "quit" if engine.update("timer:changed", |_| Ok(())).is_ok() => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                window::show(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
