# Data Model: Trigger Parameter Fix

## Entities

### `BandpassFilter` (Refactored)
| Field | Type | Description |
|-------|------|-------------|
| sections | Vec<Biquad> | Cascaded biquad sections |
| initialized | bool | State tracking |

**Methods:**
- `new(order: usize, low_freq: f64, high_freq: f64, sample_rate: f64) -> Self`
  - Calculates coefficients dynamically.

### `Biquad` (Unchanged structure, updated instantiation)
- `b0, b1, b2, a1, a2`
- `x1, x2, y1, y2` (State)

## Config Mapping

| settings.toml | TriggerConfig | Usage |
|---------------|---------------|-------|
| `alert.highpass` | `highpass` | Lower bound of passband (Hz) |
| `alert.lowpass` | `lowpass` | Upper bound of passband (Hz) |
| `alert.sta` | `sta_sec` | `n_sta = sta_sec * sample_rate` |
| `alert.lta` | `lta_sec` | `n_lta = lta_sec * sample_rate` |
