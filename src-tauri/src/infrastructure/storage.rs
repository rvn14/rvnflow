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
        let data: AppData = serde_json::from_slice(&fs::read(path)?)?;
        data.validate()?;
        Ok(data)
    }
    pub fn load(&self) -> Result<AppData> {
        let path = self.directory.join("app-state.json");
        if !path.exists() {
            return Ok(AppData::default());
        }
        match self.read(&path) {
            Ok(data) => Ok(data),
            Err(error) => {
                // Never downgrade an unknown schema or erase unrecognized user data.
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&fs::read(&path)?)
                    && value
                        .get("schemaVersion")
                        .and_then(|v| v.as_u64())
                        .is_some_and(|v| v != 1)
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
