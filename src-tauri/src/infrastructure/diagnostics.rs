use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
static DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
static WRITER: Mutex<()> = Mutex::new(());
pub fn init(directory: PathBuf) {
    let _ = fs::create_dir_all(&directory);
    let _ = DIRECTORY.set(directory);
    record("startup", "initializing");
}
pub fn record(event: &str, code: &str) {
    let Some(dir) = DIRECTORY.get() else { return };
    let Ok(_guard) = WRITER.lock() else { return };
    let path = dir.join("diagnostics.log");
    if fs::metadata(&path).is_ok_and(|m| m.len() > 256 * 1024) {
        let _ = fs::rename(&path, dir.join("diagnostics.previous.log"));
    }
    // Callers supply fixed event/error codes only. Never log task titles or file contents.
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{} {event} {code}", super::clock::wall_ms());
    }
}
