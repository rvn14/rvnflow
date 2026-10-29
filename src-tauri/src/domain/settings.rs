use crate::error::{Result, invalid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub focus_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub sessions_per_cycle: u32,
    pub long_break_enabled: bool,
    pub long_break_interval: u32,
    pub repeat_routine: bool,
    pub reduced_motion: bool,
    pub always_on_top: bool,
    pub alarm_enabled: bool,
    pub alarm: String,
    pub alarm_volume: f32,
    pub tick_enabled: bool,
    pub tick: String,
    pub tick_volume: f32,
    pub notifications: bool,
    pub start_with_windows: bool,
    pub remember_position: bool,
    pub close_to_tray: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_per_cycle: 4,
            long_break_enabled: false,
            long_break_interval: 2,
            repeat_routine: true,
            reduced_motion: false,
            always_on_top: true,
            alarm_enabled: true,
            alarm: "chime".into(),
            alarm_volume: 0.65,
            tick_enabled: false,
            tick: "mechanical".into(),
            tick_volume: 0.2,
            notifications: true,
            start_with_windows: false,
            remember_position: true,
            close_to_tray: true,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !(1..=180).contains(&self.focus_minutes)
            || !(1..=60).contains(&self.short_break_minutes)
            || !(1..=120).contains(&self.long_break_minutes)
            || !(1..=12).contains(&self.sessions_per_cycle)
            || !(1..=12).contains(&self.long_break_interval)
        {
            return Err(invalid(
                "Use focus 1–180, short break 1–60, long break 1–120 minutes and 1–12 sessions.",
            ));
        }
        if ![self.alarm_volume, self.tick_volume]
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        {
            return Err(invalid("Volume must be between 0 and 100%."));
        }
        if !["chime", "bell", "digital", "custom"].contains(&self.alarm.as_str())
            || !["mechanical", "soft"].contains(&self.tick.as_str())
        {
            return Err(invalid("Choose an available sound."));
        }
        Ok(())
    }
}
