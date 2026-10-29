use crate::{
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
        let mut data: AppData = serde_json::from_slice(&fs::read(path)?)?;
        if data.schema_version == 1 {
            data.schema_version = 2;
            // Old completed sessions were already counted; select the next without crediting again.
            if data.timer.status == crate::domain::timer::Status::Completed {
                data.timer
                    .select(data.timer.next(&data.settings), &data.settings);
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
                    .and_then(|v| v.as_u64())
                    == Some(1);
                let migration_backup = self.directory.join("app-state.v1.json");
                if legacy && !migration_backup.exists() {
                    fs::copy(&path, migration_backup)?;
                }
                Ok(data)
            }
            Err(error) => {
                // Never downgrade an unknown schema or erase unrecognized user data.
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&fs::read(&path)?)
                    && value
                        .get("schemaVersion")
                        .and_then(|v| v.as_u64())
                        .is_some_and(|v| v != 1 && v != 2)
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
