# Contract: RSAM Module

**Feature**: 035-rsam-calculation
**Date**: 2026-02-10

## Public API

### RsamManager

```rust
pub struct RsamManager { /* private fields */ }

impl RsamManager {
    /// Create from settings and sensitivity map.
    /// Binds UDP socket, validates config.
    /// Returns Err if destination address cannot be resolved.
    pub fn new(
        settings: &RsamSettings,
        sensitivity_map: HashMap<String, f64>,
    ) -> Result<Self, RsamError>;

    /// Process a trace segment from the pipeline.
    /// Filters by channel, accumulates samples, calculates and sends on interval.
    /// Non-blocking: UDP send errors are logged, never panic.
    pub fn process_segment(&mut self, segment: &TraceSegment);

    /// Force a calculation and return the result (for testing).
    /// Does not send via UDP.
    pub fn calculate(&self) -> Option<RsamResult>;
}
```

### RsamResult

```rust
pub struct RsamResult {
    pub station: String,
    pub channel: String,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

impl RsamResult {
    /// Format as LITE: stn:{station}|ch:{channel}|mean:{v}|med:{v}|min:{v}|max:{v}
    pub fn format_lite(&self) -> String;

    /// Format as JSON: {"station":"...","channel":"...","mean":...,...}
    pub fn format_json(&self) -> String;

    /// Format as CSV: {station},{channel},{mean},{median},{min},{max}
    pub fn format_csv(&self) -> String;

    /// Format using the specified format string (LITE/JSON/CSV).
    /// Unknown format falls back to LITE with warning.
    pub fn format(&self, fwformat: &str) -> String;
}
```

### RsamError

```rust
pub enum RsamError {
    /// Destination address could not be resolved
    AddressResolve(String),
    /// Failed to bind UDP socket
    SocketBind(std::io::Error),
}
```

## Integration Points

### pipeline.rs

```rust
// New parameter added to run_pipeline()
pub async fn run_pipeline(
    // ... existing parameters ...
    forward_manager: Option<Arc<ForwardManager>>,
    rsam_manager: Option<RsamManager>,  // NEW — not Arc, uses &mut self
)

// In the processing loop, after segments are parsed:
if let Some(rsam) = &mut rsam_manager {
    for seg in &segments {
        rsam.process_segment(seg);
    }
}
```

### main.rs

```rust
// Initialize RSAM manager
let rsam_manager = if settings.rsam.enabled {
    match RsamManager::new(&settings.rsam, sens_map.clone()) {
        Ok(rm) => Some(rm),
        Err(e) => {
            tracing::error!("RSAM configuration error: {}", e);
            std::process::exit(1);
        }
    }
} else {
    None
};

// Pass to pipeline (both simulation and live modes)
run_pipeline(..., forward_manager, rsam_manager).await;
```

### lib.rs

```rust
pub mod rsam;  // NEW line added
```

## Wire Format Examples

### LITE Format

```
stn:R6E01|ch:EHZ|mean:523.45|med:510.2|min:10.5|max:1050.3
```

### JSON Format

```json
{"station":"R6E01","channel":"EHZ","mean":523.45,"median":510.2,"min":10.5,"max":1050.3}
```

### CSV Format

```
R6E01,EHZ,523.45,510.2,10.5,1050.3
```

## Channel Matching

Reuses `forward::should_forward_channel()` with single-element filter list:

```rust
use crate::forward::should_forward_channel;

// In process_segment():
if !should_forward_channel(&segment.channel, &[self.settings.channel.clone()]) {
    return; // Not our channel
}
```

## Deconvolution

Applied per-sample before taking absolute value:

```rust
let converted = if self.deconvolve {
    if let Some(sens) = self.sensitivity {
        let base = sample / sens;  // Counts → physical unit
        if self.units == "GRAV" { base / 9.81 } else { base }
    } else {
        sample  // Fallback to raw counts
    }
} else {
    sample  // Raw counts
};
let abs_value = converted.abs();
```

CHAN mode resolution (done once at first matching segment):
- Channel starts with "EH" → use VEL (geophone)
- Channel starts with "EN" → use ACC (accelerometer)
- Otherwise → use VEL as default
