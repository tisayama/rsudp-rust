use std::collections::HashMap;
use std::fmt;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use tracing::{info, warn};

use crate::forward::should_forward_channel;
use crate::parser::TraceSegment;
use crate::settings::RsamSettings;

// --- Error Type ---

#[derive(Debug)]
pub enum RsamError {
    AddressResolve(String),
    SocketBind(std::io::Error),
}

impl fmt::Display for RsamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RsamError::AddressResolve(addr) => {
                write!(f, "RSAM: cannot resolve destination address '{}'", addr)
            }
            RsamError::SocketBind(e) => write!(f, "RSAM: failed to bind UDP socket: {}", e),
        }
    }
}

impl std::error::Error for RsamError {}

// --- Result Type ---

pub struct RsamResult {
    pub station: String,
    pub channel: String,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

impl RsamResult {
    pub fn format_lite(&self) -> String {
        format!(
            "stn:{}|ch:{}|mean:{}|med:{}|min:{}|max:{}",
            self.station, self.channel, self.mean, self.median, self.min, self.max
        )
    }

    pub fn format_json(&self) -> String {
        format!(
            "{{\"station\":\"{}\",\"channel\":\"{}\",\"mean\":{},\"median\":{},\"min\":{},\"max\":{}}}",
            self.station, self.channel, self.mean, self.median, self.min, self.max
        )
    }

    pub fn format_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.station, self.channel, self.mean, self.median, self.min, self.max
        )
    }

    pub fn format(&self, fwformat: &str) -> String {
        match fwformat.to_uppercase().as_str() {
            "JSON" => self.format_json(),
            "CSV" => self.format_csv(),
            "LITE" => self.format_lite(),
            other => {
                warn!("RSAM: unknown fwformat '{}', falling back to LITE", other);
                self.format_lite()
            }
        }
    }
}

// --- Manager ---

pub struct RsamManager {
    settings: RsamSettings,
    socket: UdpSocket,
    dest_addr: SocketAddr,
    buffer: Vec<f64>,
    last_calc_time: Instant,
    station: String,
    matched_channel: String,
    sensitivity: Option<f64>,
    sensitivity_map: HashMap<String, f64>,
    warm: bool,
}

impl RsamManager {
    pub fn new(
        settings: &RsamSettings,
        sensitivity_map: HashMap<String, f64>,
    ) -> Result<Self, RsamError> {
        let dest_str = format!("{}:{}", settings.fwaddr, settings.fwport);
        let dest_addr: SocketAddr = dest_str
            .parse()
            .map_err(|_| RsamError::AddressResolve(dest_str))?;

        let socket =
            UdpSocket::bind("0.0.0.0:0").map_err(RsamError::SocketBind)?;
        socket.set_nonblocking(true).ok();

        // Validate fwformat, warn on unknown
        let fmt_upper = settings.fwformat.to_uppercase();
        if !matches!(fmt_upper.as_str(), "LITE" | "JSON" | "CSV") {
            warn!(
                "RSAM: unknown fwformat '{}', will fall back to LITE",
                settings.fwformat
            );
        }

        info!(
            "RSAM: channel={}, interval={}s, format={}, destination={}, deconvolve={}, units={}",
            settings.channel,
            settings.interval,
            settings.fwformat,
            dest_addr,
            settings.deconvolve,
            settings.units
        );

        Ok(Self {
            settings: settings.clone(),
            socket,
            dest_addr,
            buffer: Vec::new(),
            last_calc_time: Instant::now(),
            station: String::new(),
            matched_channel: String::new(),
            sensitivity: None,
            sensitivity_map,
            warm: false,
        })
    }

    pub fn process_segment(&mut self, segment: &TraceSegment) {
        // Channel suffix matching
        if !should_forward_channel(
            &segment.channel,
            std::slice::from_ref(&self.settings.channel),
        ) {
            return;
        }

        // First match: resolve station, channel, sensitivity
        if !self.warm {
            self.station = segment.station.clone();
            self.matched_channel = segment.channel.clone();
            self.sensitivity = self.sensitivity_map.get(&segment.channel).copied();

            // CHAN mode: resolve units based on channel prefix
            if self.settings.units.to_uppercase() == "CHAN" {
                let ch_upper = segment.channel.to_uppercase();
                if ch_upper.starts_with("EH") {
                    // Geophone → VEL
                } else if ch_upper.starts_with("EN") {
                    // Accelerometer → ACC
                }
                // For CHAN mode, we just use sample/sensitivity regardless
            }

            self.warm = true;
        }

        // Accumulate absolute amplitude samples (with optional deconvolution)
        let deconvolve = self.settings.deconvolve;
        let units_upper = self.settings.units.to_uppercase();
        let sensitivity = self.sensitivity;

        for &sample in &segment.samples {
            let converted = if deconvolve {
                if let Some(sens) = sensitivity {
                    let base = sample / sens;
                    if units_upper == "GRAV" {
                        base / 9.81
                    } else {
                        base
                    }
                } else {
                    sample // Fallback to raw counts
                }
            } else {
                sample // Raw counts
            };
            self.buffer.push(converted.abs());
        }

        // Check if interval has elapsed
        let interval = Duration::from_secs(self.settings.interval as u64);
        if self.last_calc_time.elapsed() >= interval {
            if let Some(result) = self.calculate() {
                // Log if not quiet
                if !self.settings.quiet {
                    info!(
                        "RSAM [{}]: mean={:.2}, median={:.2}, min={:.2}, max={:.2}",
                        result.channel, result.mean, result.median, result.min, result.max
                    );
                }

                // Format and send via UDP
                let formatted = result.format(&self.settings.fwformat);
                if let Err(e) = self.socket.send_to(formatted.as_bytes(), self.dest_addr) {
                    warn!("RSAM: UDP send error: {}", e);
                }
            }

            // Reset buffer and timer
            self.buffer.clear();
            self.last_calc_time = Instant::now();
        }
    }

    pub fn calculate(&self) -> Option<RsamResult> {
        if self.buffer.is_empty() {
            return None;
        }

        let len = self.buffer.len() as f64;
        let mean = self.buffer.iter().sum::<f64>() / len;

        let mut sorted = self.buffer.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if sorted.len().is_multiple_of(2) {
            let mid = sorted.len() / 2;
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let min = sorted.first().copied().unwrap_or(0.0);
        let max = sorted.last().copied().unwrap_or(0.0);

        Some(RsamResult {
            station: self.station.clone(),
            channel: self.matched_channel.clone(),
            mean,
            median,
            min,
            max,
        })
    }
}
