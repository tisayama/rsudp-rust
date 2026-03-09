use rodio::{OutputStream, Sink, Decoder};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::sync::mpsc;
use std::path::Path;
use tracing::{info, error};

/// AudioManager that queues audio playback requests and plays them
/// sequentially on a dedicated thread.
///
/// Each file is played with a fresh ALSA output stream to avoid
/// "alsa::poll() returned POLLERR" errors after hours of idle on
/// Raspberry Pi.
pub struct AudioManager {
    sender: mpsc::Sender<String>,
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

        let (sender, receiver) = mpsc::channel::<String>();

        std::thread::spawn(move || {
            while let Ok(file_path) = receiver.recv() {
                play_file_internal(&file_path);
            }
            // All Senders dropped → recv() returns Err → thread exits
        });

        info!("AudioManager initialized (queued playback mode)");
        Some(Self { sender })
    }

    /// Queue an audio file for sequential playback.
    /// Empty file paths are ignored (FR-004).
    /// This method is non-blocking (FR-006).
    pub fn queue_file(&self, file_path: &str) {
        if file_path.is_empty() {
            return;
        }
        if let Err(e) = self.sender.send(file_path.to_string()) {
            error!("Failed to queue audio file: {}", e);
        }
    }
}

/// Internal playback function executed on the dedicated playback thread.
/// Creates a fresh ALSA OutputStream per file (FR-003, Raspberry Pi stability).
/// Handles missing/invalid files gracefully (FR-005).
fn play_file_internal(file_path: &str) {
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
    sink.sleep_until_end();
}

pub type AudioController = Arc<AudioManager>;
