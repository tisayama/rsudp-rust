# Data Model: rsudp-style Alert Post Timing

## Entities

### 1. AlertTask (Internal)
Represents a scheduled notification and image generation task.

| Field | Type | Description |
|-------|------|-------------|
| alert_id | Uuid | ID of the triggering alert |
| channel | String | Channel name |
| trigger_time | DateTime | Actual time of the trigger |
| execute_at | Instant | Wall-clock time to run the task |
| initial_max_intensity | f64 | Peak intensity at the time of trigger |

### 2. PlotConfig (Updated)
Configuration parameters for plotting.

| Field | Type | Description |
|-------|------|-------------|
| duration | f64 | Seconds of data to display in plot (default: 90) |
| save_pct | f64 | Ratio of duration to wait before posting (default: 0.7) |

## Logic Flow

1. **Detection (Trigger)**:
   - `TriggerManager` emits `Trigger`.
   - Pipeline calculates `delay = config.duration * config.save_pct`.
   - Pipeline spawns a background task:
     - `sleep(delay).await`
     - Extract last `config.duration` seconds from `waveform_buffers`.
     - Calculate max intensity in that window.
     - Generate PNG and send Email/WebSocket summary.

2. **Detection (Reset)**:
   - `TriggerManager` emits `Reset`.
   - Pipeline updates internal `active_alerts` state.
   - **NO** notification or image generation happens here.
