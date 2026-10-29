use crate::{
    error::{AppError, Result},
    state::AppData,
};
use tauri::{LogicalSize, Manager, PhysicalPosition};
fn native(error: tauri::Error) -> AppError {
    AppError::Native(error.to_string())
}
pub fn apply(app: &tauri::AppHandle, data: &AppData, restore: bool) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        // CSS draws the only rounded surface. A native shadow adds an extra border
        // and halo on Windows, while an opaque webview fills the transparent corners.
        window.set_shadow(false).map_err(native)?;
        window
            .set_always_on_top(data.settings.always_on_top)
            .map_err(native)?;
        let size = if data.window.expanded {
            (350.0, 560.0)
        } else {
            (210.0, 105.0)
        };
        window
            .set_size(LogicalSize::new(size.0, size.1))
            .map_err(native)?;
        if restore
            && data.settings.remember_position
            && let Some((x, y)) = data.window.position
        {
            window
                .set_position(PhysicalPosition::new(x, y))
                .map_err(native)?;
        }
        clamp(&window)?;
    }
    Ok(())
}
pub fn clamp(window: &tauri::WebviewWindow) -> Result<()> {
    let position = window.outer_position().map_err(native)?;
    let size = window.outer_size().map_err(native)?;
    let monitors = window.available_monitors().map_err(native)?;
    // Select the nearest work area; this also recovers when a display disappears.
    if let Some(monitor) = monitors.iter().min_by_key(|m| {
        let area = m.work_area();
        let dx = i64::from(position.x)
            - i64::from(position.x).clamp(
                i64::from(area.position.x),
                i64::from(area.position.x) + i64::from(area.size.width),
            );
        let dy = i64::from(position.y)
            - i64::from(position.y).clamp(
                i64::from(area.position.y),
                i64::from(area.position.y) + i64::from(area.size.height),
            );
        dx * dx + dy * dy
    }) {
        let area = monitor.work_area();
        let x = position.x.clamp(
            area.position.x,
            area.position.x + (area.size.width.saturating_sub(size.width)) as i32,
        );
        let y = position.y.clamp(
            area.position.y,
            area.position.y + (area.size.height.saturating_sub(size.height)) as i32,
        );
        if x != position.x || y != position.y {
            window
                .set_position(PhysicalPosition::new(x, y))
                .map_err(native)?;
        }
    }
    Ok(())
}
pub fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = clamp(&window);
        let _ = window.set_skip_taskbar(false);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
pub fn hide(app: &tauri::AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(native)?;
        window.set_skip_taskbar(true).map_err(native)?;
    }
    Ok(())
}
