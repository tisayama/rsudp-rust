# Data Model: STA/LTA Alert System

## Entities

### AlertConfig
Configuration parameters for the alert system.

| Field | Type | Description |
|-------|------|-------------|
| `sta_seconds` | `f64` | Short-Term Average window length in seconds. |
| `lta_seconds` | `f64` | Long-Term Average window length in seconds. |
| `threshold` | `f64` | STA/LTA ratio to trigger an ALARM. |
| `reset_threshold` | `f64` | STA/LTA ratio to return to MONITORING. |
| `min_duration` | `f64` | Optional: Minimum time (s) threshold must be exceeded to trigger. |
| `channel_id` | `String` | The channel being monitored (e.g., "SHZ"). |
| `filter_config` | `Option<FilterConfig>` | Optional digital filter settings. |

### FilterConfig
Settings for the preprocessing filter.

| Field | Type | Description |
|-------|------|-------------|
| `filter_type` | `Enum` | `Bandpass`, `Highpass`, `Lowpass`, or `None`. |
| `freq_min` | `f64` | Lower corner frequency (Hz). |
| `freq_max` | `f64` | Upper corner frequency (Hz). |
| `order` | `usize` | Filter order (default: 4, matching ObsPy). |

### AlertState
Internal state of the monitoring process.

| State | Description |
|-------|-------------|
| `WarmingUp` | System is accumulating data for the LTA window. |
| `Monitoring` | System is calculating STA/LTA and waiting for trigger. |
| `Alarm` | Threshold exceeded; system is in alarm state. |

### AlertEvent
Metadata emitted when a state change occurs.

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | `Enum` | `Alarm` or `Reset`. |
| `timestamp` | `DateTime<Utc>` | Time of the event. |
| `channel_id` | `String` | Channel that triggered the event. |
| `max_ratio` | `Option<f64>` | Maximum STA/LTA ratio reached (included in `Reset`). |

## State Transitions

1.  **Initialize** -> `WarmingUp`
2.  `WarmingUp` -> (after `lta_seconds`) -> `Monitoring`
3.  `Monitoring` -> (ratio > `threshold`) -> `Alarm`
4.  `Alarm` -> (ratio < `reset_threshold`) -> `Monitoring`
5.  **Any State** -> (Gap detected) -> `WarmingUp`
