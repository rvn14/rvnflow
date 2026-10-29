use crate::{
    domain::timer::Status,
    error::Result,
    infrastructure::{
        clock::{now_ms, wall_ms},
        diagnostics, notifications,
        storage::Storage,
    },
    services::{audio::Audio, repository::Repository},
    state::{AppData, Completion},
};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub struct Engine {
    pub repository: Repository,
    pub storage: Storage,
    pub audio: Audio,
    pub app: tauri::AppHandle,
    signal: Mutex<u64>,
    wake: Condvar,
    pub position: Mutex<Option<(i32, i32)>>,
}
impl Engine {
    pub fn new(app: tauri::AppHandle, storage: Storage) -> Result<Arc<Self>> {
        let mut data = storage.load()?;
        data.timer.recover(now_ms(), wall_ms());
        let audio = Audio::new(app.clone(), storage.directory.join("custom-audio"));
        Ok(Arc::new(Self {
            repository: Repository::new(data),
            storage,
            audio,
            app,
            signal: Mutex::new(0),
            wake: Condvar::new(),
            position: Mutex::new(None),
        }))
    }
    pub fn snapshot(&self) -> Result<AppData> {
        Ok(self.repository.snapshot()?.snapshot(now_ms()))
    }
    pub fn update(
        &self,
        event: &str,
        operation: impl FnOnce(&mut AppData) -> Result<()>,
    ) -> Result<AppData> {
        // Closures only mutate domain data. Commands perform OS effects afterwards.
        let previous = self.repository.snapshot()?;
        let mut completion = None;
        let result = self.repository.transact(
            |d| {
                if let Some(finished) = d.timer.advance(now_ms(), wall_ms(), &d.settings) {
                    let record = Completion {
                        finished_at: wall_ms(),
                        session: finished,
                        task_id: d.timer.task_id.clone(),
                        next_session: Some(d.timer.session),
                    };
                    d.history.push(record.clone());
                    if d.history.len() > 1000 {
                        d.history.remove(0);
                    }
                    completion = Some(record);
                }
                operation(d)?;
                if d.timer.status == Status::Running {
                    d.timer.remaining_ms = d.timer.remaining(now_ms());
                    d.timer.wall_deadline_ms = Some(wall_ms().saturating_add(d.timer.remaining_ms));
                }
                Ok(())
            },
            |d| self.storage.save(d),
        );
        let data = match result {
            Ok(data) => data,
            Err(error) => {
                diagnostics::record("transaction", "failed");
                return Err(error);
            }
        };
        // No repository writer/data locks survive into notifications, events or audio.
        let snapshot = data.snapshot(now_ms());
        if let Some(record) = completion {
            self.completed(&snapshot, &record);
        }
        if previous.settings.tick_enabled
            && (!data.settings.tick_enabled || data.timer.status != Status::Running)
        {
            self.audio.stop_ticks();
        }
        let _ = self.app.emit(event, &snapshot);
        if let Ok(mut generation) = self.signal.lock() {
            *generation = generation.wrapping_add(1);
            self.wake.notify_one();
        }
        Ok(snapshot)
    }
    fn completed(&self, data: &AppData, record: &Completion) {
        diagnostics::record("timer", "completed");
        self.audio.stop_ticks();
        let _ = self.app.emit("timer:completed", data);
        if data.settings.alarm_enabled
            && self
                .audio
                .play(&data.settings.alarm, data.settings.alarm_volume, false)
                .is_err()
        {
            self.report("Audio playback is unavailable.", "alarm");
        }
        let foreground = self.app.get_webview_window("main").is_some_and(|w| {
            w.is_visible().unwrap_or(false)
                && !w.is_minimized().unwrap_or(false)
                && w.is_focused().unwrap_or(false)
        });
        if data.settings.notifications && !foreground {
            let (title, body) = notifications::content(record, &data.settings);
            if self
                .app
                .notification()
                .builder()
                .title(title)
                .body(body)
                .show()
                .is_err()
            {
                self.report(
                    "Windows could not show a notification. Check Windows notification settings.",
                    "notification",
                );
            }
        }
    }
    pub fn report(&self, message: &str, code: &str) {
        diagnostics::record(code, "failed");
        let _ = self.app.emit("app:error", message);
    }
    pub fn persist_position(&self) -> Result<()> {
        let position = self.position.lock().ok().and_then(|mut p| p.take());
        if let Some(position) = position {
            self.update("window:changed", |d| {
                if d.settings.remember_position {
                    d.window.position = Some(position);
                }
                Ok(())
            })?;
        }
        Ok(())
    }
    pub fn spawn(engine: Arc<Self>) {
        std::thread::spawn(move || {
            let mut last_tick = None;
            let mut last_boot = now_ms();
            let mut last_wall = wall_ms();
            loop {
                let generation = match engine.signal.lock() {
                    Ok(g) => *g,
                    Err(_) => return,
                };
                let Ok(data) = engine.snapshot() else { return };
                let running = data.timer.status == Status::Running;
                if running && data.timer.remaining_ms == 0 {
                    if engine.update("timer:changed", |_| Ok(())).is_err() {
                        engine.report(
                            "Unable to save session completion. Check your disk and retry.",
                            "completion-save",
                        );
                        std::thread::sleep(Duration::from_secs(5));
                    }
                    continue;
                }
                if running && data.settings.tick_enabled {
                    let second = (data.timer.duration_ms - data.timer.remaining_ms) / 1000;
                    let key = (data.timer.deadline, second);
                    if second > 0 && last_tick != Some(key) {
                        let _ =
                            engine
                                .audio
                                .play(&data.settings.tick, data.settings.tick_volume, true);
                        last_tick = Some(key);
                    }
                } else {
                    last_tick = None;
                    engine.audio.stop_ticks();
                }
                let boot = now_ms();
                let wall = wall_ms();
                if running
                    && (boot.saturating_sub(last_boot) > 2000
                        || (wall as i128
                            - last_wall as i128
                            - boot.saturating_sub(last_boot) as i128)
                            .abs()
                            > 2000)
                {
                    let _ = engine.app.emit("timer:changed", &data);
                }
                last_boot = boot;
                last_wall = wall;
                let Ok(signal) = engine.signal.lock() else {
                    return;
                };
                if *signal != generation {
                    continue;
                }
                if running {
                    let delay = data.timer.remaining_ms % 1000;
                    if engine
                        .wake
                        .wait_timeout(
                            signal,
                            Duration::from_millis(if delay == 0 { 1000 } else { delay }),
                        )
                        .is_err()
                    {
                        return;
                    }
                } else if engine.wake.wait(signal).is_err() {
                    return;
                }
            }
        });
    }
}
