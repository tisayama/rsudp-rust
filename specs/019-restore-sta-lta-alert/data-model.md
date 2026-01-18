# Data Model: Trigger State

## Entities

### `StaLtaState`
Persistent state for a single channel.

| Field | Type | Description |
|-------|------|-------------|
| sta | f64 | Short Term Average (recursive) |
| lta | f64 | Long Term Average (recursive) |
| filter | BandpassFilter | IIR filter state |
| last_timestamp | Option<DateTime<Utc>> | Timestamp of last processed sample |

### `BandpassFilter`
Cascaded Biquad filter state.

| Field | Type | Description |
|-------|------|-------------|
| sections | Vec<Biquad> | Filter stages |
| initialized | bool | Has the first sample been seen? |

## Logic Flow

1. **Input**: New chunk of samples.
2. **Loop**: For each sample:
   - Check timestamp continuity (reset if >1s jump).
   - Apply filter (updates `x1, x2, y1, y2`).
   - Update STA/LTA (recursive formula).
   - Check `sta/lta > threshold`.
   - Update trigger status.
