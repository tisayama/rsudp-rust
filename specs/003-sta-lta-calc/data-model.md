# Data Model: STA/LTA Calculation

## Entities

### `RecursiveStaLta` (Struct)

Holds the state for the recursive calculation.

| Field | Type | Description |
|---|---|---|
| `csta` | `f64` | Coefficient for STA update (`1.0 / nsta`). |
| `clta` | `f64` | Coefficient for LTA update (`1.0 / nlta`). |
| `sta` | `f64` | Current Short Term Average value. |
| `lta` | `f64` | Current Long Term Average value. |

## Input/Output

- **Input**: `f64` (sample amplitude) or `&[f64]` (chunk of samples)
- **Output**: `f64` (current ratio) or `Vec<f64>` (ratios for the chunk)
