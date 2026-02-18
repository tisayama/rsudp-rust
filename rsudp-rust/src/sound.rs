use rodio::{OutputStream, Sink, Decoder};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::path::Path;
use tracing::{info, error};

/// AudioManager that creates a fresh ALSA output stream per playback.
///
/// Long-lived OutputStream instances on Raspberry Pi accumulate ALSA
/// buffer errors ("alsa::poll() returned POLLERR") after hours of idle.
/// By opening the stream only when needed and closing it after playback
/// completes, we avoid this problem entirely.
pub struct AudioManager {
    _marker: (), // No persistent state needed
}

impl AudioManager {
    pub fn new() -> Option<Self> {
        // Verify that an audio device exists at startup
        match OutputStream::try_default() {
            Ok(_) => {
                info!("Audio output device detected");
                // Drop the test stream immediately
            }
            Err(e) => {
                error!("No audio output device available: {}", e);
                return None;
            }
        }

        Some(Self { _marker: () })
    }

    pub fn play_file(&self, file_path: &str) {
        let path = Path::new(file_path);
        if !path.exists() {
            error!("Audio file not found: {}", file_path);
            return;
        }

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open audio file {}: {}", file_path, e);
                return;
            }
        };

        let source = match Decoder::new(BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to decode audio file {}: {}", file_path, e);
                return;
            }
        };

        // Open a fresh output stream for this playback
        let (_stream, handle) = match OutputStream::try_default() {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to open audio stream: {}", e);
                return;
            }
        };

        let sink = match Sink::try_new(&handle) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create audio sink: {}", e);
                return;
            }
        };

        sink.append(source);
        info!("Playing audio: {}", file_path);

        // Block until playback completes, then drop stream + sink.
        // This runs on spawn_blocking so it won't block the tokio runtime.
        sink.sleep_until_end();
    }
}

pub type AudioController = Arc<AudioManager>;
