use crate::domain::{settings::Settings, task::Tasks, timer::Timer};
use crate::error::{Result, invalid};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub expanded: bool,
    pub position: Option<(i32, i32)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub finished_at: u64,
    pub session: crate::domain::timer::Session,
    pub task_id: Option<String>,
    #[serde(default)]
    pub next_session: Option<crate::domain::timer::Session>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub schema_version: u32,
    pub revision: u64,
    pub settings: Settings,
    pub timer: Timer,
    pub tasks: Tasks,
    pub window: WindowState,
    pub history: Vec<Completion>,
    pub custom_audio: bool,
    #[serde(default, skip_deserializing)]
    pub recovery: Option<String>,
    #[serde(default, skip_deserializing)]
    pub sampled_at: u64,
}
impl Default for AppData {
    fn default() -> Self {
        Self {
            schema_version: 3,
            revision: 0,
            settings: Settings::default(),
            timer: Timer::default(),
            tasks: Tasks::default(),
            window: WindowState::default(),
            history: vec![],
            custom_audio: false,
            recovery: None,
            sampled_at: 0,
        }
    }
}
impl AppData {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 3 {
            return Err(invalid(
                "This data uses an unsupported schema. Install a compatible RVNFlow version.",
            ));
        }
        self.settings.validate()?;
        self.tasks.validate()?;
        self.timer.validate()?;
        if let Some(id) = &self.timer.task_id
            && !self.tasks.tasks.iter().any(|t| &t.id == id)
        {
            return Err(invalid("The current task is missing."));
        }
        Ok(())
    }
    pub fn snapshot(&self, now: u64) -> Self {
        let mut copy = self.clone();
        copy.timer = self.timer.view(now);
        copy.sampled_at = now;
        copy
    }
}
