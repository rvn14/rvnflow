//! Debug-only native integration harness. It never uses the user's data directory.
use crate::{
    error::{Result, invalid},
    infrastructure::diagnostics,
    services::{engine::Engine, window},
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::Manager;
pub fn start(engine: Arc<Engine>) {
    if std::env::var_os("RVNFLOW_NATIVE_SMOKE").is_none() {
        return;
    }
    std::thread::spawn(move || {
        let result = exercise(&engine);
        let message = match result {
            Ok(()) => "passed".to_owned(),
            Err(error) => format!("failed: {error}"),
        };
        let _ = std::fs::write(engine.storage.directory.join("smoke-result.txt"), message);
        engine.app.exit(0);
    });
}
fn exercise(engine: &Engine) -> Result<()> {
    let wait = Instant::now();
    while !std::fs::read_to_string(engine.storage.directory.join("diagnostics.log"))
        .unwrap_or_default()
        .contains("frontend ready")
    {
        if wait.elapsed() > Duration::from_secs(15) {
            return Err(invalid("frontend did not connect to native IPC"));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let window = engine
        .app
        .get_webview_window("main")
        .ok_or_else(|| invalid("missing window"))?;
    engine.update("timer:changed", |d| {
        d.timer.start(
            crate::infrastructure::clock::now_ms(),
            crate::infrastructure::clock::wall_ms(),
            &d.settings,
        );
        Ok(())
    })?;
    for expanded in [true, false, true, false] {
        let data = engine.update("window:changed", |d| {
            d.window.expanded = expanded;
            Ok(())
        })?;
        window::apply(&engine.app, &data, false)?;
        let size = window.inner_size().map_err(|e| invalid(e.to_string()))?;
        let logical =
            size.to_logical::<u32>(window.scale_factor().map_err(|e| invalid(e.to_string()))?);
        if logical.width != if expanded { 350 } else { 210 }
            || logical.height != if expanded { 560 } else { 105 }
        {
            return Err(invalid("mode size did not match frontend mode"));
        }
    }
    window.minimize().map_err(|e| invalid(e.to_string()))?;
    std::thread::sleep(Duration::from_millis(250));
    if !window.is_minimized().map_err(|e| invalid(e.to_string()))? {
        return Err(invalid("native minimize failed"));
    }
    window::show(&engine.app);
    if window.is_minimized().map_err(|e| invalid(e.to_string()))? {
        return Err(invalid("restore stayed minimized"));
    }
    window::hide(&engine.app)?;
    if window.is_visible().map_err(|e| invalid(e.to_string()))? {
        return Err(invalid("tray hide stayed visible"));
    }
    window::show(&engine.app);
    if !window.is_visible().map_err(|e| invalid(e.to_string()))? {
        return Err(invalid("tray restore stayed hidden"));
    }
    if engine.snapshot()?.timer.status != crate::domain::timer::Status::Running {
        return Err(invalid("timer stopped while minimized or hidden"));
    }
    engine.update("tasks:changed", |d| {
        if d.tasks.tasks.is_empty() {
            let list = d.tasks.active_list_id.clone();
            d.tasks.create(&list, "Native smoke fixture", 1)?;
        }
        d.settings.notifications = false;
        d.settings.alarm_enabled = false;
        Ok(())
    })?;
    let history = engine.snapshot()?.history.len();
    engine.update("timer:changed", |d| {
        d.timer
            .select(crate::domain::timer::Session::Focus, &d.settings);
        d.timer.start(
            crate::infrastructure::clock::now_ms(),
            crate::infrastructure::clock::wall_ms(),
            &d.settings,
        );
        d.timer.deadline = Some(crate::infrastructure::clock::now_ms());
        Ok(())
    })?;
    let complete_wait = Instant::now();
    while engine.snapshot()?.history.len() == history {
        if complete_wait.elapsed() > Duration::from_secs(3) {
            return Err(invalid("native completion did not run"));
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    let completed = engine.snapshot()?;
    if completed.timer.session != crate::domain::timer::Session::ShortBreak
        || completed.timer.status != crate::domain::timer::Status::Idle
        || completed.timer.remaining_ms != 300_000
    {
        return Err(invalid("native focus did not select ready short break"));
    }
    std::thread::sleep(Duration::from_millis(100));
    if engine.snapshot()?.history.len() != history + 1 {
        return Err(invalid("duplicate native history record"));
    }
    diagnostics::record("smoke", "await-second-instance");
    std::fs::write(engine.storage.directory.join("smoke-ready.txt"), "ready")?;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        let log = std::fs::read_to_string(engine.storage.directory.join("diagnostics.log"))
            .unwrap_or_default();
        if log.contains("activate-existing") {
            engine.update("timer:changed", |_| Ok(()))?;
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(invalid(
        "second instance did not activate the existing process",
    ))
}
