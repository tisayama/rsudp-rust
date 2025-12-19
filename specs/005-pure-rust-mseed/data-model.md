# Data Model: Pure Rust MiniSEED Ingestion

## Entities

### `SeedHeader` (Internal)

Binary mapping of the 48-byte Fixed Section Data Header.

| Field | Type | Bits/Bytes |
|---|---|---|
| `station` | `String` | 5 bytes |
| `location` | `String` | 2 bytes |
| `channel` | `String` | 3 bytes |
| `network` | `String` | 2 bytes |
| `starttime` | `DateTime<Utc>` | 10 bytes (BTIME) |
| `num_samples` | `u16` | 2 bytes |
| `sample_rate` | `f64` | Calculated from factor/multiplier |

### `SteimFrame` (Internal)

A 64-byte block of compressed differences.

| Field | Type | Description |
|---|---|---|
| `control_word` | `u32` | 16 * 2-bit flags |
| `words` | `[u32; 15]` | Data words containing differences |

## Compatibility

The parser output MUST be convertible to the following public entity:

### `TraceSegment` (Shared with 004)

- `network`: String
- `station`: String
- `location`: String
- `channel`: String
- `starttime`: DateTime<Utc>
- `sampling_rate`: f64
- `samples`: Vec<f64>
