# Data Model: WebUI Plot System

## Entities

### WaveformPacket (WebSocket Message)
Real-time data chunk sent from backend to frontend.

| Field | Type | Description |
|-------|------|-------------|
| `channel_id` | `String` | ID of the seismic channel (e.g., "SHZ"). |
| `timestamp` | `DateTime<Utc>` | Start timestamp of the first sample in the packet. |
| `samples` | `Vec<f32>` | Batch of seismic samples (raw or counts). |
| `sample_rate` | `f32` | Frequency of samples in Hz. |

### PlotSettings (REST API Entity)
User-specific or global configuration for the display.

| Field | Type | Description |
|-------|------|-------------|
| `active_channels` | `Vec<String>` | List of channel IDs to display. |
| `window_seconds` | `u32` | History duration to show on screen (e.g., 60, 300). |
| `auto_scale` | `bool` | Whether to automatically adjust Y-axis height. |
| `theme` | `String` | UI theme (dark/light). |

### AlertEvent (WebSocket/REST Entity)
Seismic trigger notification.

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | `Enum` | `ALARM` or `RESET`. |
| `timestamp` | `DateTime<Utc>` | When the event occurred. |
| `channel_id` | `String` | Which channel triggered the alert. |
| `max_ratio` | `Option<f32>` | Peak STA/LTA ratio (sent with `RESET`). |

## State Transitions

### Frontend Buffer State
1. **Empty**: No data received yet.
2. **Buffering**: Accumulating initial samples to fill the time window.
3. **Full/Scrolling**: Window filled; oldest samples discarded as new ones arrive.

### WebSocket Connection State
1. **Disconnected**: Attempting initial connection.
2. **Connected**: Streaming data.
3. **Reconnecting**: Connection lost; exponential backoff in progress.
