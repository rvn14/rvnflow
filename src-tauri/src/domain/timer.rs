use super::settings::Settings;
use crate::error::{Result, invalid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Session {
    Focus,
    ShortBreak,
    LongBreak,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub session: Session,
    pub status: Status,
    pub duration_ms: u64,
    pub remaining_ms: u64,
    pub completed_focus: u32,
    #[serde(default)]
    pub routine_completed_focus: u32,
    #[serde(default)]
    pub routine_complete: bool,
    #[serde(default)]
    pub focus_since_long_break: u32,
    pub task_id: Option<String>,
    pub wall_deadline_ms: Option<u64>,
    #[serde(skip)]
    pub deadline: Option<u64>,
}
impl Default for Timer {
    fn default() -> Self {
        Self::new(&Settings::default())
    }
}
impl Timer {
    pub fn new(settings: &Settings) -> Self {
        Self {
            session: Session::Focus,
            status: Status::Idle,
            duration_ms: settings.focus_minutes as u64 * 60_000,
            remaining_ms: settings.focus_minutes as u64 * 60_000,
            completed_focus: 0,
            routine_completed_focus: 0,
            routine_complete: false,
            focus_since_long_break: 0,
            task_id: None,
            wall_deadline_ms: None,
            deadline: None,
        }
    }
    pub fn duration(session: Session, settings: &Settings) -> u64 {
        (match session {
            Session::Focus => settings.focus_minutes,
            Session::ShortBreak => settings.short_break_minutes,
            Session::LongBreak => settings.long_break_minutes,
        }) as u64
            * 60_000
    }
    pub fn remaining(&self, now: u64) -> u64 {
        if self.status == Status::Running {
            self.deadline
                .map_or(self.remaining_ms, |d| d.saturating_sub(now))
                .min(self.duration_ms)
        } else {
            self.remaining_ms
        }
    }
    pub fn view(&self, now: u64) -> Self {
        let mut view = self.clone();
        view.remaining_ms = self.remaining(now);
        view
    }
    pub fn validate(&self) -> Result<()> {
        if !(60_000..=10_800_000).contains(&self.duration_ms)
            || self.remaining_ms > self.duration_ms
            || (self.status == Status::Running && self.wall_deadline_ms.is_none())
            || (self.status == Status::Completed && self.remaining_ms != 0)
        {
            return Err(invalid(
                "Invalid saved timer. Your task data has been preserved.",
            ));
        }
        Ok(())
    }
    pub fn recover(&mut self, now: u64, wall: u64) {
        if self.status == Status::Running {
            // Restart cannot reconstruct monotonic time. Clamp wall-clock recovery to
            // the saved remaining time, so moving the clock backward never adds time.
            self.remaining_ms = self
                .wall_deadline_ms
                .unwrap_or(wall)
                .saturating_sub(wall)
                .min(self.remaining_ms);
            self.deadline = Some(now.saturating_add(self.remaining_ms));
            self.wall_deadline_ms = Some(wall.saturating_add(self.remaining_ms));
        }
    }
    pub fn select(&mut self, session: Session, settings: &Settings) {
        self.session = session;
        self.status = Status::Idle;
        self.duration_ms = Self::duration(session, settings);
        self.remaining_ms = self.duration_ms;
        self.deadline = None;
        self.wall_deadline_ms = None;
        self.routine_complete = false;
        if session == Session::LongBreak {
            self.focus_since_long_break = 0;
        }
    }
    pub fn next(&self, settings: &Settings) -> Session {
        match self.session {
            Session::Focus
                if settings.long_break_enabled
                    && self.focus_since_long_break >= settings.long_break_interval =>
            {
                Session::LongBreak
            }
            Session::Focus => Session::ShortBreak,
            _ => Session::Focus,
        }
    }
    pub fn start(&mut self, now: u64, wall: u64, settings: &Settings) {
        if self.status == Status::Running {
            return;
        }
        if self.status == Status::Completed {
            if self.routine_complete {
                self.routine_completed_focus = 0;
                self.focus_since_long_break = 0;
                self.select(Session::Focus, settings);
            } else {
                self.select(self.next(settings), settings);
            }
        }
        self.status = Status::Running;
        self.deadline = Some(now.saturating_add(self.remaining_ms));
        self.wall_deadline_ms = Some(wall.saturating_add(self.remaining_ms));
    }
    pub fn pause(&mut self, now: u64) {
        if self.status == Status::Running {
            self.remaining_ms = self.remaining(now);
            self.status = Status::Paused;
            self.deadline = None;
            self.wall_deadline_ms = None;
        }
    }
    pub fn complete(&mut self, now: u64) -> bool {
        if self.status != Status::Running || self.remaining(now) != 0 {
            return false;
        }
        self.status = Status::Completed;
        self.remaining_ms = 0;
        self.deadline = None;
        self.wall_deadline_ms = None;
        if self.session == Session::Focus {
            self.completed_focus = self.completed_focus.saturating_add(1);
            self.routine_completed_focus = self.routine_completed_focus.saturating_add(1);
            self.focus_since_long_break = self.focus_since_long_break.saturating_add(1);
        }
        true
    }
    pub fn advance(&mut self, now: u64, wall: u64, settings: &Settings) -> Option<Session> {
        let finished = self.session;
        if !self.complete(now) {
            return None;
        }
        if finished == Session::Focus
            && self.routine_completed_focus >= settings.sessions_per_cycle
            && !settings.repeat_routine
        {
            self.routine_complete = true;
            return Some(finished);
        }
        let next = self.next(settings);
        self.select(next, settings);
        if next == Session::Focus && self.routine_completed_focus >= settings.sessions_per_cycle {
            self.routine_completed_focus = 0;
        }
        // Every scheduled phase starts automatically. Anchor to the current wake
        // time, not the missed deadline, so suspend cannot consume the whole routine.
        self.start(now, wall, settings);
        Some(finished)
    }
    pub fn skip(&mut self, now: u64, wall: u64, settings: &Settings) {
        if self.routine_complete {
            return;
        }
        // A skipped focus always leads to a short break and never earns a cycle credit.
        let status = self.status;
        let next = if self.session == Session::Focus {
            Session::ShortBreak
        } else {
            Session::Focus
        };
        self.select(next, settings);
        if next == Session::Focus
            && self.routine_completed_focus >= settings.sessions_per_cycle
            && settings.repeat_routine
        {
            self.routine_completed_focus = 0;
        }
        if status == Status::Running {
            self.start(now, wall, settings);
        } else if status == Status::Paused {
            self.status = Status::Paused;
        }
    }
}
