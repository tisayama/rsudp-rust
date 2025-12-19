# Data Model: Data Ingestion Pipeline

## Entities

### `TraceSegment`

A contiguous sequence of samples from a single source.

| Field | Type | Description |
|---|---|---|
| `network` | `String` | Network code (e.g., "AM"). |
| `station` | `String` | Station code (e.g., "R1234"). |
| `location` | `String` | Location ID (e.g., "00"). |
| `channel` | `String` | Channel code (e.g., "SHZ"). |
| `starttime` | `DateTime<Utc>` | Start time of the segment. |
| `sampling_rate` | `f64` | Samples per second. |
| `samples` | `Vec<f64>` | The numerical amplitude data. |

## Pipeline Flow

1.  **UDP Packet / File Content** (`Vec<u8>`)
2.  **Parser** -> produces 1 or more `TraceSegment`s.
3.  **Processor** -> looks up `RecursiveStaLta` by NSLC, checks for gaps, then calls `process_chunk`.
