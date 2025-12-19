# Data Model: Japanese Seismic Intensity Calculation

## Entities

### IntensityConfig
Configuration for the intensity calculation module.

| Field | Type | Description |
|-------|------|-------------|
| `channels` | `[String; 3]` | The 3 channel IDs used for 3-axis calculation (e.g., ENE, ENN, ENZ). |
| `sample_rate` | `f64` | Sampling rate in Hz (must be identical for all 3 channels). |
| `sensitivities` | `[f64; 3]` | Conversion factors (Counts per Gal) for each channel. |

### IntensityResult
The output of the calculation.

| Field | Type | Description |
|-------|------|-------------|
| `instrumental_intensity` | `f64` | The raw calculated value `I`. |
| `intensity_class` | `String` | Human-readable class (e.g., "3", "5 Lower"). |
| `timestamp` | `DateTime<Utc>` | The end time of the 60s window used for calculation. |

### IntensityState (Internal)
Internal buffer for the sliding window.

| Field | Type | Description |
|-------|------|-------------|
| `buffers` | `HashMap<String, VecDeque<f64>>` | 60-second history of raw samples for each configured channel. |
| `last_calculation` | `DateTime<Utc>` | Last time the 1s update was performed. |

## Intensity Class Mapping

| Instrumental Intensity `I` | JMA Intensity Class |
|---------------------------|---------------------|
| `I < 0.5`                 | `0`                 |
| `0.5 <= I < 1.5`          | `1`                 |
| `1.5 <= I < 2.5`          | `2`                 |
| `2.5 <= I < 3.5`          | `3`                 |
| `3.5 <= I < 4.5`          | `4`                 |
| `4.5 <= I < 5.0`          | `5 Lower`           |
| `5.0 <= I < 5.5`          | `5 Upper`           |
| `5.5 <= I < 6.0`          | `6 Lower`           |
| `6.0 <= I < 6.5`          | `6 Upper`           |
| `6.5 <= I`                | `7`                 |
