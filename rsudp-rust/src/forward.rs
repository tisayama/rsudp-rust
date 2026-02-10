use std::fmt;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::settings::ForwardSettings;

// --- Types ---

/// Message sent from pipeline to forwarding task.
#[derive(Debug, Clone)]
pub enum ForwardMsg {
    /// Raw seismic data packet bytes to forward.
    Data(Vec<u8>),
    /// ALARM or RESET event text message.
    Alarm(String),
}

/// Errors during ForwardManager initialization.
#[derive(Debug)]
pub enum ForwardError {
    /// address and port list lengths do not match.
    ConfigMismatch { addresses: usize, ports: usize },
    /// Failed to bind UDP socket.
    SocketBind(std::io::Error),
    /// Failed to resolve destination address.
    AddressResolve(String),
}

impl fmt::Display for ForwardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForwardError::ConfigMismatch { addresses, ports } => {
                write!(f, "Forward config: address count ({}) != port count ({})", addresses, ports)
            }
            ForwardError::SocketBind(e) => write!(f, "Forward socket bind error: {}", e),
            ForwardError::AddressResolve(addr) => write!(f, "Forward: cannot resolve address '{}'", addr),
        }
    }
}

impl std::error::Error for ForwardError {}

/// Per-destination forwarding statistics (atomic for concurrent access).
struct ForwardStats {
    packets_sent: AtomicU64,
    packets_dropped: AtomicU64,
    send_errors: AtomicU64,
}

impl ForwardStats {
    fn new() -> Self {
        Self {
            packets_sent: AtomicU64::new(0),
            packets_dropped: AtomicU64::new(0),
            send_errors: AtomicU64::new(0),
        }
    }
}

/// A single forwarding destination with its mpsc sender.
struct ForwardDestination {
    addr: SocketAddr,
    tx: mpsc::Sender<ForwardMsg>,
    stats: Arc<ForwardStats>,
}

/// Manages UDP packet forwarding to one or more remote destinations.
pub struct ForwardManager {
    destinations: Vec<ForwardDestination>,
    channels: Vec<String>,
    fwd_data: bool,
    fwd_alarms: bool,
}

impl fmt::Debug for ForwardManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForwardManager")
            .field("destinations", &self.destinations.len())
            .field("fwd_data", &self.fwd_data)
            .field("fwd_alarms", &self.fwd_alarms)
            .finish()
    }
}

// --- Channel Matching ---

/// Check if a channel name matches the configured filter list.
/// - "all" matches everything.
/// - Otherwise, case-insensitive suffix matching (e.g., filter "HZ" matches "EHZ").
/// - Empty filters or no matches fall back to matching all (with warning on first call).
pub fn should_forward_channel(channel: &str, filters: &[String]) -> bool {
    if filters.is_empty() {
        return true;
    }
    let channel_upper = channel.to_uppercase();
    for filter in filters {
        let filter_upper = filter.to_uppercase();
        if filter_upper == "ALL" {
            return true;
        }
        if channel_upper.ends_with(&filter_upper) {
            return true;
        }
    }
    false
}

// --- ForwardManager Implementation ---

impl ForwardManager {
    /// Create from settings, validate config, bind sockets, spawn forwarding tasks.
    pub async fn new(settings: &ForwardSettings) -> Result<Self, ForwardError> {
        // FR-006: Validate address/port length match
        if settings.address.len() != settings.port.len() {
            return Err(ForwardError::ConfigMismatch {
                addresses: settings.address.len(),
                ports: settings.port.len(),
            });
        }

        let mut destinations = Vec::with_capacity(settings.address.len());

        for (i, (addr_str, &port)) in settings.address.iter().zip(&settings.port).enumerate() {
            // Resolve destination address
            let dest_addr: SocketAddr = format!("{}:{}", addr_str, port)
                .parse()
                .map_err(|_| ForwardError::AddressResolve(format!("{}:{}", addr_str, port)))?;

            // Bind local UDP socket (port 0 = OS-assigned ephemeral port)
            let socket = UdpSocket::bind("0.0.0.0:0")
                .await
                .map_err(ForwardError::SocketBind)?;

            // Bounded channel (capacity 32) per destination
            let (tx, rx) = mpsc::channel::<ForwardMsg>(32);
            let stats = Arc::new(ForwardStats::new());

            // Spawn async forwarding task for this destination
            let task_id = i;
            let task_addr = dest_addr;
            let task_stats = stats.clone();
            tokio::spawn(async move {
                run_forward_task(task_id, task_addr, socket, rx, task_stats).await;
            });

            destinations.push(ForwardDestination {
                addr: dest_addr,
                tx,
                stats,
            });
        }

        // Startup confirmation log (T021 / FR-008)
        let addr_list: Vec<String> = destinations.iter().map(|d| d.addr.to_string()).collect();
        info!(
            "Forward: {} destination(s) configured [{}]",
            destinations.len(),
            addr_list.join(", ")
        );
        info!(
            "Forward: channels={:?}, fwd_data={}, fwd_alarms={}",
            settings.channels, settings.fwd_data, settings.fwd_alarms
        );

        Ok(Self {
            destinations,
            channels: settings.channels.clone(),
            fwd_data: settings.fwd_data,
            fwd_alarms: settings.fwd_alarms,
        })
    }

    /// Send a raw data packet for forwarding (if fwd_data enabled).
    /// Filters by channel. Non-blocking: drops packet if queue full.
    pub fn forward_data(&self, channel: &str, raw_packet: &[u8]) {
        if !self.fwd_data {
            return;
        }
        if !should_forward_channel(channel, &self.channels) {
            return;
        }
        let msg = ForwardMsg::Data(raw_packet.to_vec());
        for dest in &self.destinations {
            if dest.tx.try_send(msg.clone()).is_err() {
                dest.stats.packets_dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Send an alarm/reset event for forwarding (if fwd_alarms enabled).
    /// Non-blocking: drops message if queue full.
    pub fn forward_alarm(&self, message: &str) {
        if !self.fwd_alarms {
            return;
        }
        let msg = ForwardMsg::Alarm(message.to_string());
        for dest in &self.destinations {
            if dest.tx.try_send(msg.clone()).is_err() {
                dest.stats.packets_dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Graceful shutdown: drop all senders to signal task termination.
    pub async fn shutdown(self) {
        info!("Forward: shutting down {} destination(s)", self.destinations.len());
        // Dropping self drops all tx senders, causing recv loops to exit.
        drop(self);
    }
}

// --- Per-Destination Forwarding Task ---

async fn run_forward_task(
    id: usize,
    addr: SocketAddr,
    socket: UdpSocket,
    mut rx: mpsc::Receiver<ForwardMsg>,
    stats: Arc<ForwardStats>,
) {
    let mut stats_interval = tokio::time::interval(Duration::from_secs(60));
    // The first tick completes immediately; skip it.
    stats_interval.tick().await;

    let mut last_sent: u64 = 0;
    let mut last_dropped: u64 = 0;
    let mut last_errors: u64 = 0;

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(forward_msg) => {
                        let bytes = match &forward_msg {
                            ForwardMsg::Data(data) => data.as_slice(),
                            ForwardMsg::Alarm(text) => text.as_bytes(),
                        };
                        match socket.send_to(bytes, addr).await {
                            Ok(_) => {
                                stats.packets_sent.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => {
                                stats.send_errors.fetch_add(1, Ordering::Relaxed);
                                warn!("Forward #{} ({}): send error: {}", id, addr, e);
                            }
                        }
                    }
                    None => {
                        // All senders dropped — shutdown
                        info!("Forward #{} ({}): task stopped", id, addr);
                        break;
                    }
                }
            }
            _ = stats_interval.tick() => {
                let sent = stats.packets_sent.load(Ordering::Relaxed);
                let dropped = stats.packets_dropped.load(Ordering::Relaxed);
                let errors = stats.send_errors.load(Ordering::Relaxed);

                let d_sent = sent - last_sent;
                let d_dropped = dropped - last_dropped;
                let d_errors = errors - last_errors;

                info!(
                    "Forward #{} ({}): sent={}, dropped={}, errors={} (delta: +{}s/+{}d/+{}e)",
                    id, addr, sent, dropped, errors, d_sent, d_dropped, d_errors
                );

                last_sent = sent;
                last_dropped = dropped;
                last_errors = errors;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_forward_channel_all() {
        assert!(should_forward_channel("EHZ", &["all".to_string()]));
        assert!(should_forward_channel("EHN", &["ALL".to_string()]));
        assert!(should_forward_channel("anything", &["All".to_string()]));
    }

    #[test]
    fn test_should_forward_channel_suffix_match() {
        let filters = vec!["HZ".to_string()];
        assert!(should_forward_channel("EHZ", &filters));
        assert!(should_forward_channel("SHZ", &filters));
        assert!(should_forward_channel("BHZ", &filters));
        assert!(!should_forward_channel("EHN", &filters));
        assert!(!should_forward_channel("EHE", &filters));
    }

    #[test]
    fn test_should_forward_channel_case_insensitive() {
        let filters = vec!["hz".to_string()];
        assert!(should_forward_channel("EHZ", &filters));
        assert!(should_forward_channel("ehz", &filters));
    }

    #[test]
    fn test_should_forward_channel_exact_match() {
        let filters = vec!["EHZ".to_string()];
        assert!(should_forward_channel("EHZ", &filters));
        assert!(!should_forward_channel("EHN", &filters));
    }

    #[test]
    fn test_should_forward_channel_multiple_filters() {
        let filters = vec!["EHZ".to_string(), "EHN".to_string()];
        assert!(should_forward_channel("EHZ", &filters));
        assert!(should_forward_channel("EHN", &filters));
        assert!(!should_forward_channel("EHE", &filters));
    }

    #[test]
    fn test_should_forward_channel_empty_filters() {
        assert!(should_forward_channel("EHZ", &[]));
        assert!(should_forward_channel("anything", &[]));
    }
}
