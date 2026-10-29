use crate::error::{Result, invalid};
use rodio::{Decoder, Source, buffer::SamplesBuffer};
use std::{
    collections::HashMap,
    io::Cursor,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    time::{Duration, Instant},
};

enum Request {
    Play(String, f32, bool, u64),
    StopTicks,
    Import(Vec<u8>, mpsc::Sender<Result<()>>),
}
pub struct Audio {
    sender: SyncSender<Request>,
}
pub fn decode(bytes: Vec<u8>) -> Result<SamplesBuffer> {
    if bytes.len() > 20 * 1024 * 1024 {
        return Err(invalid("Choose a WAV or MP3 file smaller than 20 MB."));
    }
    let decoder = Decoder::try_from(Cursor::new(bytes))
        .map_err(|_| invalid("This file is not a decodable WAV or MP3 sound."))?;
    let channels = decoder.channels();
    let rate = decoder.sample_rate();
    if channels.get() > 2 || rate.get() > 192_000 {
        return Err(invalid("Choose mono or stereo audio at 192 kHz or below."));
    }
    // Bound decoded memory as well as compressed file size (maximum 30 seconds).
    let limit = usize::from(channels.get()) * rate.get() as usize * 30;
    let samples: Vec<f32> = decoder.take(limit + 1).collect();
    if samples.is_empty() || samples.len() > limit {
        return Err(invalid(
            "Choose a nonempty sound no longer than 30 seconds.",
        ));
    }
    Ok(SamplesBuffer::new(channels, rate, samples))
}
impl Audio {
    pub fn new(app: tauri::AppHandle, custom_path: std::path::PathBuf) -> Self {
        let (sender, receiver) = mpsc::sync_channel(8);
        std::thread::spawn(move || {
            use tauri::Emitter;
            let mut cache = HashMap::new();
            for (key, bytes) in [
                ("chime", include_bytes!("../../sounds/chime.wav").as_slice()),
                ("bell", include_bytes!("../../sounds/bell.wav").as_slice()),
                (
                    "digital",
                    include_bytes!("../../sounds/digital.wav").as_slice(),
                ),
                (
                    "mechanical",
                    include_bytes!("../../sounds/mechanical.wav").as_slice(),
                ),
                ("soft", include_bytes!("../../sounds/soft.wav").as_slice()),
            ] {
                if let Ok(sample) = decode(bytes.to_vec()) {
                    cache.insert(key.to_owned(), sample);
                }
            }
            if let Ok(bytes) = std::fs::read(custom_path)
                && let Ok(sample) = decode(bytes)
            {
                cache.insert("custom".into(), sample);
            }
            let mut device = None;
            let mut alarm: Option<rodio::Player> = None;
            let mut tick: Option<rodio::Player> = None;
            let mut last_use = Instant::now();
            let failed = Arc::new(AtomicBool::new(false));
            let mut retry_after = None;
            loop {
                let request = if device.is_some() {
                    receiver.recv_timeout(Duration::from_millis(250))
                } else {
                    receiver
                        .recv()
                        .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
                };
                match request {
                    Ok(Request::Import(bytes, reply)) => {
                        let result = decode(bytes).map(|sample| {
                            cache.insert("custom".into(), sample);
                        });
                        let _ = reply.send(result);
                    }
                    Ok(Request::StopTicks) => {
                        tick = None;
                    }
                    Ok(Request::Play(name, volume, is_tick, created)) => {
                        if failed.swap(false, Ordering::Relaxed) {
                            alarm = None;
                            tick = None;
                            device = None;
                        }
                        if is_tick && retry_after.is_some_and(|time| Instant::now() < time) {
                            continue;
                        }
                        // Late ticks are discarded; never replay a backlog after suspend.
                        if volume <= 0.0
                            || (is_tick
                                && crate::infrastructure::clock::now_ms().saturating_sub(created)
                                    > 200)
                        {
                            continue;
                        }
                        let Some(sample) = cache.get(&name) else {
                            let _=app.emit("app:error","The selected sound is unavailable. Import it again or choose a built-in sound.");
                            continue;
                        };
                        if device.is_none() {
                            let failure_flag = failed.clone();
                            let output = rodio::DeviceSinkBuilder::from_default_device().and_then(
                                |builder| {
                                    builder
                                        .with_error_callback(move |_| {
                                            failure_flag.store(true, Ordering::Relaxed);
                                        })
                                        .open_sink_or_fallback()
                                },
                            );
                            match output {
                                Ok(mut stream) => {
                                    stream.log_on_drop(false);
                                    device = Some(stream);
                                    retry_after = None;
                                }
                                Err(error) => {
                                    retry_after = Some(Instant::now() + Duration::from_secs(10));
                                    let _ = app.emit(
                                        "app:error",
                                        format!("Audio output unavailable: {error}"),
                                    );
                                    continue;
                                }
                            }
                        }
                        if let Some(stream) = &device {
                            let player = rodio::Player::connect_new(stream.mixer());
                            player.set_volume(volume);
                            player.append(sample.clone());
                            if is_tick {
                                tick = Some(player);
                            } else {
                                alarm = Some(player);
                            }
                            last_use = Instant::now();
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if alarm.as_ref().is_none_or(|p| p.empty())
                    && last_use.elapsed() > Duration::from_millis(1200)
                {
                    // Reopen on demand, so device changes recover and idle audio uses no stream.
                    alarm = None;
                    tick = None;
                    device = None;
                }
                // Keep the current tick alive until playback ends.
                if tick.as_ref().is_some_and(|p| p.empty()) {
                    tick = None;
                }
            }
        });
        Self { sender }
    }
    pub fn play(&self, name: &str, volume: f32, tick: bool) -> Result<()> {
        self.sender
            .try_send(Request::Play(
                name.into(),
                volume,
                tick,
                crate::infrastructure::clock::now_ms(),
            ))
            .map_err(|_| invalid("Audio is busy. Please try again."))
    }
    pub fn stop_ticks(&self) {
        let _ = self.sender.try_send(Request::StopTicks);
    }
    pub fn import(&self, path: &Path, destination: &Path) -> Result<()> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !["wav", "mp3"].contains(&extension.as_str()) {
            return Err(invalid("Choose a WAV or MP3 file."));
        }
        if std::fs::metadata(path)?.len() > 20 * 1024 * 1024 {
            return Err(invalid("The sound must be smaller than 20 MB."));
        }
        let bytes = std::fs::read(path)?;
        decode(bytes.clone())?;
        let parent = destination
            .parent()
            .ok_or_else(|| invalid("Audio data folder is unavailable."))?;
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        use std::io::Write;
        temp.write_all(&bytes)?;
        temp.as_file().sync_all()?;
        temp.persist(destination).map_err(|e| e.error)?;
        let (tx, rx) = mpsc::channel();
        self.sender
            .send(Request::Import(bytes, tx))
            .map_err(|_| invalid("Audio service is unavailable."))?;
        rx.recv_timeout(Duration::from_secs(10))
            .map_err(|_| invalid("Audio import timed out."))?
    }
}
