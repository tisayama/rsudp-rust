# Data Model: UDP MiniSEED Streamer

## Entities

### 1. RecordIndexEntry
Represents a metadata entry for a single MiniSEED record found in the input file.

| Field | Type | Description |
|-------|------|-------------|
| start_time | DateTime<Utc> | Absolute start time of the record from header |
| file_offset | u64 | Byte offset in the file where the record starts |
| length | u32 | Length of the record (usually 512 bytes) |
| channel | String | NSLC (Network.Station.Location.Channel) string |
| sample_rate | f64 | Sampling rate in Hz |
| num_samples | u32 | Number of samples in the record |

### 2. StreamerConfig
Configuration for the streaming session.

| Field | Type | Description |
|-------|------|-------------|
| file_path | PathBuf | Path to the source MiniSEED file |
| target_addr | SocketAddr | UDP destination (IP:Port) |
| speed_multiplier | f64 | Playback speed (1.0 = real-time) |
| loop_mode | bool | Whether to restart from beginning |
| buffer_size | usize | Size of the internal playback queue |

### 3. PlaybackState
Internal state tracking the current simulation progress.

| Field | Type | Description |
|-------|------|-------------|
| session_start_real | Instant | Wall-clock time when simulation started |
| session_start_data | DateTime<Utc> | Data-time of the first record in simulation |
| current_record_idx | usize | Index of the next record to send |

## State Transitions

1. **Indexing Phase**: Read file sequentially -> Extract headers -> Populate `Vec<RecordIndexEntry>`.
2. **Sorting Phase**: Sort `Vec<RecordIndexEntry>` by `start_time` (Ascending).
3. **Streaming Phase**:
   - Calculate delay: `(RecordStartTime - SessionStartData) / SpeedMultiplier`.
   - Wait until `Instant::now() >= SessionStartReal + Delay`.
   - Read 512 bytes from disk.
   - Decode Steim2 samples.
   - Serialize to rsudp string format.
   - Send via UDP.
4. **Looping**: If `LoopMode` and end reached -> Reset `PlaybackState`.
