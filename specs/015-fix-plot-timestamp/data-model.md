# Data Model: rsudp Plot Timestamp & Grid Fix

## Logic Enhancements

### 1. Time Label Formatting
The X-axis label generation will follow this logic:
- Input: `x` (f64, seconds from start)
- Ref: `start_time` (DateTime<Utc>)
- Output: `(start_time + x).format("%H:%M:%S")`

### 2. View Composition
- **Waveform Area**:
  - Mesh: Disabled
  - X-Axis: Disabled (Labels hidden)
  - Y-Axis: Enabled (Counts/Units shown)
- **Spectrogram Area**:
  - Mesh: Disabled
  - X-Axis: Enabled (UTC timestamps shown)
  - Y-Axis: Enabled (Frequency shown)
