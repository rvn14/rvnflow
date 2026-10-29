use crate::{
    domain::timer::Status,
    error::{Result, invalid},
    infrastructure::{
        clock::{now_ms, wall_ms},
        storage::Storage,
    },
    services::audio::Audio,
    state::{AppData, Completion},
};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;

pub struct Engine {
    pub data: Mutex<AppData>,
    pub wake: Condvar,
    pub storage: Storage,
    pub audio: Audio,
    pub app: tauri::AppHandle,
}
impl Engine {
    pub fn new(app: tauri::AppHandle, storage: Storage) -> Result<Arc<Self>> {
        let mut data = storage.load()?;
        data.timer.recover(now_ms(), wall_ms());
        let audio = Audio::new(app.clone(), storage.directory.join("custom-audio"));
        let engine = Arc::new(Self {
            data: Mutex::new(data),
            wake: Condvar::new(),
            storage,
            audio,
            app,
        });
        // Persist recovered completion before exposing the first snapshot.
        engine.update("timer:changed", |_| Ok(()))?;
        Ok(engine)
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, AppData>> {
        self.data
            .lock()
            .map_err(|_| invalid("The application state is unavailable. Restart RVNFlow."))
    }
    pub fn snapshot(&self) -> Result<AppData> {
        Ok(self.lock()?.snapshot(now_ms()))
    }
    pub fn update(
        &self,
        event: &str,
        operation: impl FnOnce(&mut AppData) -> Result<()>,
    ) -> Result<AppData> {
        let mut guard = self.lock()?;
        let mut candidate = guard.clone();
        let completed = candidate.timer.complete(now_ms());
        if completed {
            candidate.history.push(Completion {
                finished_at: wall_ms(),
                session: candidate.timer.session,
                task_id: candidate.timer.task_id.clone(),
            });
            // Bound history, never tasks. History is informational in version 1.
            if candidate.history.len() > 1000 {
                candidate.history.remove(0);
            }
            self.commit(&mut guard, candidate.clone())?;
            self.completed(&guard);
        }
        operation(&mut candidate)?;
        self.commit(&mut guard, candidate)?;
        let snapshot = guard.snapshot(now_ms());
        let _ = self.app.emit(event, &snapshot);
        self.wake.notify_one();
        Ok(snapshot)
    }
    fn commit(&self, current: &mut AppData, mut next: AppData) -> Result<()> {
        next.revision = current.revision.saturating_add(1);
        // Save a fresh wall deadline without modifying the boot-clock deadline.
        if next.timer.status == Status::Running {
            next.timer.remaining_ms = next.timer.remaining(now_ms());
            next.timer.wall_deadline_ms = Some(wall_ms().saturating_add(next.timer.remaining_ms));
        }
        self.storage.save(&next)?;
        *current = next;
        Ok(())
    }
    fn completed(&self, data: &AppData) {
        self.audio.stop_ticks();
        let _ = self.app.emit("timer:completed", data.snapshot(now_ms()));
        if data.settings.alarm_enabled
            && let Err(error) =
                self.audio
                    .play(&data.settings.alarm, data.settings.alarm_volume, false)
        {
            let _ = self.app.emit("app:error", error.to_string());
        }
        if data.settings.notifications
            && let Err(error) = self
                .app
                .notification()
                .builder()
                .title("Session complete")
                .body("A moment to breathe. Start your next session when you are ready.")
                .show()
        {
            let _ = self
                .app
                .emit("app:error", format!("Notification unavailable: {error}"));
        }
    }
    pub fn spawn(engine: Arc<Self>) {
        std::thread::spawn(move || {
            let mut last_tick = None;
            let mut last_boot = now_ms();
            let mut last_wall = wall_ms();
            loop {
                let Ok(guard) = engine.lock() else { return };
                let running = guard.timer.status == Status::Running;
                let remaining = guard.timer.remaining(now_ms());
                if running && remaining == 0 {
                    drop(guard);
                    if let Err(error) = engine.update("timer:changed", |_| Ok(())) {
                        let _ = engine.app.emit("app:error", error.to_string());
                        std::thread::sleep(Duration::from_secs(5));
                    }
                    continue;
                }
                if running && guard.settings.tick_enabled {
                    let elapsed = guard.timer.duration_ms.saturating_sub(remaining) / 1000;
                    let key = (guard.timer.deadline, elapsed);
                    if elapsed > 0 && last_tick != Some(key) {
                        let _ = engine.audio.play(
                            &guard.settings.tick,
                            guard.settings.tick_volume,
                            true,
                        );
                        last_tick = Some(key);
                    }
                } else {
                    last_tick = None;
                    engine.audio.stop_ticks();
                }
                let boot = now_ms();
                let wall = wall_ms();
                if running
                    && ((boot.saturating_sub(last_boot) > 2000)
                        || (wall as i128
                            - last_wall as i128
                            - boot.saturating_sub(last_boot) as i128)
                            .abs()
                            > 2000)
                {
                    // Resynchronize rendering after suspend or a system-clock change, not every second.
                    let _ = engine.app.emit("timer:changed", guard.snapshot(boot));
                }
                last_boot = boot;
                last_wall = wall;
                if running {
                    let delay = remaining % 1000;
                    let result = engine.wake.wait_timeout(
                        guard,
                        Duration::from_millis(if delay == 0 { 1000 } else { delay }),
                    );
                    if result.is_err() {
                        return;
                    }
                } else if engine.wake.wait(guard).is_err() {
                    return;
                }
            }
        });
    }
}
