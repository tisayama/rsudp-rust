# Contract: Forward Module (src/forward.rs)

**Date**: 2026-02-10

## Public API

### `ForwardManager`

```rust
pub struct ForwardManager { /* private */ }

impl ForwardManager {
    /// Create from settings, validate config, bind sockets.
    /// Returns Err if address/port lengths mismatch or socket binding fails.
    pub async fn new(settings: &ForwardSettings) -> Result<Self, ForwardError>;

    /// Send a raw data packet for forwarding (if fwd_data enabled).
    /// Filters by channel. Non-blocking: drops packet if queue full.
    /// `channel` is the parsed channel name (e.g., "EHZ").
    pub fn forward_data(&self, channel: &str, raw_packet: &[u8]);

    /// Send an alarm/reset event for forwarding (if fwd_alarms enabled).
    /// Non-blocking: drops message if queue full.
    pub fn forward_alarm(&self, message: &str);

    /// Graceful shutdown: signal all forwarding tasks to stop.
    pub async fn shutdown(&self);
}
```

### `ForwardError`

```rust
pub enum ForwardError {
    /// address and port list lengths do not match
    ConfigMismatch { addresses: usize, ports: usize },
    /// Failed to bind UDP socket
    SocketBind(std::io::Error),
    /// Failed to resolve destination address
    AddressResolve(String),
}
```

## Integration Points

### Pipeline (caller)

```rust
// In run_pipeline(), after parse_any():
if let Some(fwd) = &forward_manager {
    if settings.forward.fwd_data {
        for segment in &segments {
            fwd.forward_data(&segment.channel, &raw_bytes);
        }
    }
}

// On AlertEventType::Trigger:
if let Some(fwd) = &forward_manager {
    fwd.forward_alarm(&format!("ALARM {} {}", segment.channel, timestamp));
}

// On AlertEventType::Reset:
if let Some(fwd) = &forward_manager {
    fwd.forward_alarm(&format!("RESET {} {}", segment.channel, timestamp));
}
```

### Main (initialization)

```rust
// In main(), after loading settings:
let forward_manager = if settings.forward.enabled {
    match ForwardManager::new(&settings.forward).await {
        Ok(fm) => {
            info!("Forward: {} destinations configured", settings.forward.address.len());
            Some(Arc::new(fm))
        }
        Err(e) => {
            error!("Forward configuration error: {}", e);
            std::process::exit(1);
        }
    }
} else {
    None
};

// Pass to run_pipeline() as new parameter
```

## Wire Format

### Data Packets (fwd_data)

Raw bytes forwarded as-is. No transformation. Receivers get the exact same bytes that arrived on the listening port.

### Alarm Messages (fwd_alarms)

UTF-8 encoded strings:
- Trigger: `ALARM {channel} {ISO-8601 timestamp}\n`
- Reset: `RESET {channel} {ISO-8601 timestamp}\n`

Example:
```
ALARM EHZ 2026-02-10T12:34:56.789Z
RESET EHZ 2026-02-10T12:35:01.234Z
```
