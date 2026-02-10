use rodio::{OutputStream, OutputStreamHandle, Sink, Decoder};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::path::Path;
use tracing::{info, error};

pub struct AudioManager {
    // _stream is removed to make struct Send. 
    // The caller must keep the OutputStream alive.
    handle: OutputStreamHandle,
    current_sink: Mutex<Option<Sink>>,
}

impl AudioManager {
    pub fn new() -> Option<(Self, OutputStream)> {
        // Try to get default output device
        // On headless servers or docker, this might fail if no audio device.
        let (_stream, handle) = match OutputStream::try_default() {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to initialize audio output stream: {}", e);
                return None;
            }
        };

        let manager = Self {
            handle,
            current_sink: Mutex::new(None),
        };
        
        Some((manager, _stream))
    }

    pub fn play_file(&self, file_path: &str) {
        let path = Path::new(file_path);
        if !path.exists() {
            error!("Audio file not found: {}", file_path);
            return;
        }

        // Load file first to ensure it's readable
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

        // Create new sink (this is cheap)
        let sink = match Sink::try_new(&self.handle) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create audio sink: {}", e);
                return;
            }
        };

        sink.append(source);
        
        // Replace current sink to stop previous sound (Preemption)
        let mut guard = self.current_sink.lock().unwrap();
        // Dropping the old sink stops playback immediately
        *guard = Some(sink);
        
        info!("Playing audio: {}", file_path);
        // We don't sleep here, we let it play in background thread managed by rodio
    }
}

// Global or shared controller wrapper if needed, but Arc<AudioManager> is sufficient
pub type AudioController = Arc<AudioManager>;
