use crate::{
    domain::timer::{Session, Status},
    error::{Result, invalid},
    state::AppData,
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct Storage {
    pub directory: PathBuf,
}
impl Storage {
    pub fn new(directory: PathBuf) -> Result<Self> {
        fs::create_dir_all(&directory)?;
        Ok(Self { directory })
    }
    fn read(&self, path: &Path) -> Result<AppData> {
        let raw: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        let mut data: AppData = serde_json::from_value(raw.clone())?;
        if data.schema_version == 1 || data.schema_version == 2 {
            data.schema_version = 3;
            // Preserve the old long-break cadence, while removing the old opt-in
            // auto-start flags: scheduled focus/break transitions are now automatic.
            if raw["settings"].get("longBreakEnabled").is_none() {
                data.settings.long_break_enabled = true;
                data.settings.long_break_interval = data.settings.sessions_per_cycle;
            }
            data.timer.routine_completed_focus =
                data.timer.completed_focus % data.settings.sessions_per_cycle.max(1);
            data.timer.focus_since_long_break =
                data.timer.completed_focus % data.settings.long_break_interval.max(1);
            // Completed legacy phases were already credited. Resume their next
            // scheduled phase without recording or alarming for the old phase again.
            let completed_focus = data.timer.status == Status::Completed
                && data.timer.session == Session::Focus
                && data.timer.completed_focus > 0;
            if completed_focus && data.timer.routine_completed_focus == 0 {
                data.timer.routine_completed_focus = data.settings.sessions_per_cycle;
            }
            if completed_focus && data.timer.focus_since_long_break == 0 {
                data.timer.focus_since_long_break = data.settings.long_break_interval;
            }
            if data.timer.status == Status::Completed {
                if completed_focus
                    && data.timer.routine_completed_focus >= data.settings.sessions_per_cycle
                    && !data.settings.repeat_routine
                {
                    data.timer.routine_complete = true;
                } else {
                    data.timer
                        .select(data.timer.next(&data.settings), &data.settings);
                    data.timer.start(
                        super::clock::now_ms(),
                        super::clock::wall_ms(),
                        &data.settings,
                    );
                }
            } else if data.timer.status == Status::Idle
                && data.timer.session != Session::Focus
                && data.history.last().is_some_and(|record| {
                    record.session == Session::Focus
                        && record.next_session == Some(data.timer.session)
                })
            {
                // v0.2 left automatically prepared breaks idle when auto-start was off.
                data.timer.start(
                    super::clock::now_ms(),
                    super::clock::wall_ms(),
                    &data.settings,
                );
            }
        }
        data.validate()?;
        Ok(data)
    }
    pub fn load(&self) -> Result<AppData> {
        let path = self.directory.join("app-state.json");
        if !path.exists() {
            let backup = self.directory.join("app-state.backup.json");
            if backup.exists() {
                let mut data = self.read(&backup)?;
                data.recovery = Some(
                    "The primary save was missing. Restored your previous saved state.".into(),
                );
                self.atomic(&path, &serde_json::to_vec_pretty(&data)?)?;
                return Ok(data);
            }
            return Ok(AppData::default());
        }
        match self.read(&path) {
            Ok(data) => {
                let legacy = serde_json::from_slice::<serde_json::Value>(&fs::read(&path)?)?
                    .get("schemaVersion")
                    .and_then(|v| v.as_u64());
                if let Some(version @ (1 | 2)) = legacy {
                    let migration_backup =
                        self.directory.join(format!("app-state.v{version}.json"));
                    if !migration_backup.exists() {
                        fs::copy(&path, migration_backup)?;
                    }
                }
                Ok(data)
            }
            Err(error) => {
                // Never downgrade an unknown schema or erase unrecognized user data.
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&fs::read(&path)?)
                    && value
                        .get("schemaVersion")
                        .and_then(|v| v.as_u64())
                        .is_some_and(|v| v != 1 && v != 2 && v != 3)
                {
                    return Err(error);
                }
                let mut data = self
                    .read(&self.directory.join("app-state.backup.json"))
                    .map_err(|_| {
                        invalid(format!(
                            "{error} No valid backup exists. Your files are untouched at {}.",
                            self.directory.display()
                        ))
                    })?;
                let quarantine = self
                    .directory
                    .join(format!("app-state.corrupt-{}.json", uuid::Uuid::new_v4()));
                fs::copy(&path, quarantine)?;
                data.recovery=Some("Recovered the previous saved state. The damaged file was preserved in your data folder.".into());
                // Replace only the malformed primary; retain the known-good backup.
                self.atomic(&path, &serde_json::to_vec_pretty(&data)?)?;
                Ok(data)
            }
        }
    }
    fn atomic(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        let mut temporary = tempfile::NamedTempFile::new_in(&self.directory)?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist(path).map_err(|e| e.error)?;
        Ok(())
    }
    pub fn save(&self, data: &AppData) -> Result<()> {
        data.validate()?;
        let path = self.directory.join("app-state.json");
        if path.exists() {
            let previous = self.read(&path)?;
            self.atomic(
                &self.directory.join("app-state.backup.json"),
                &serde_json::to_vec_pretty(&previous)?,
            )?;
        }
        self.atomic(&path, &serde_json::to_vec_pretty(data)?)
    }
}
