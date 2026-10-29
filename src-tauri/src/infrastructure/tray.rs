use crate::{
    domain::timer::Status,
    infrastructure::clock::{now_ms, wall_ms},
    services::{startup::Startup, window},
};
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
pub fn install(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show RVNFlow", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide to tray", true, None::<&str>)?;
    let timer = MenuItem::with_id(app, "timer", "Start / pause session", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit RVNFlow", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &timer, &quit])?;
    let mut tray = TrayIconBuilder::new()
        .tooltip("RVNFlow · A little space to focus")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let app = app.clone();
            let id = event.id.as_ref().to_owned();
            tauri::async_runtime::spawn_blocking(move || {
                if id == "show" {
                    window::show(&app);
                    return;
                }
                let state = app.state::<Startup>();
                let Ok(engine) = state.ready() else {
                    if id == "quit" {
                        app.exit(0);
                    }
                    return;
                };
                match id.as_str() {
                    "hide" => {
                        let _ = engine.persist_position();
                        let _ = window::hide(&app);
                    }
                    "timer" => {
                        if engine
                            .update("timer:changed", |d| {
                                if d.timer.status == Status::Running {
                                    d.timer.pause(now_ms());
                                } else {
                                    d.timer.start(now_ms(), wall_ms(), &d.settings);
                                }
                                Ok(())
                            })
                            .is_err()
                        {
                            engine.report(
                                "Unable to save the timer. Please try again.",
                                "tray-timer",
                            );
                        }
                    }
                    "quit" => {
                        if engine
                            .persist_position()
                            .and_then(|_| engine.update("window:changed", |_| Ok(())))
                            .is_ok()
                        {
                            app.exit(0);
                        } else {
                            engine.report("Unable to save before quitting.", "quit-save");
                        }
                    }
                    _ => {}
                }
            });
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
                let app = tray.app_handle().clone();
                tauri::async_runtime::spawn_blocking(move || window::show(&app));
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
