use crate::{
    error::{Result, invalid},
    state::AppData,
};
use std::sync::{Mutex, RwLock};

/// Serializes writers without making snapshots or UI callbacks wait for filesystem I/O.
pub struct Repository {
    data: RwLock<AppData>,
    writer: Mutex<()>,
}
impl Repository {
    pub fn new(data: AppData) -> Self {
        Self {
            data: RwLock::new(data),
            writer: Mutex::new(()),
        }
    }
    pub fn snapshot(&self) -> Result<AppData> {
        Ok(self
            .data
            .read()
            .map_err(|_| invalid("Application state is unavailable."))?
            .clone())
    }
    pub fn transact(
        &self,
        operation: impl FnOnce(&mut AppData) -> Result<()>,
        save: impl FnOnce(&AppData) -> Result<()>,
    ) -> Result<AppData> {
        let _writer = self
            .writer
            .lock()
            .map_err(|_| invalid("The save service is unavailable."))?;
        let mut candidate = self.snapshot()?;
        operation(&mut candidate)?;
        candidate.revision = candidate.revision.saturating_add(1);
        candidate.validate()?;
        // No data lock is held during the operation or disk I/O. Window callbacks
        // can always read the last committed snapshot, even during a slow save.
        save(&candidate)?;
        *self
            .data
            .write()
            .map_err(|_| invalid("Application state is unavailable."))? = candidate.clone();
        Ok(candidate)
    }
}
