# Data Model: Streaming Logic

## Entities

### `ChunkConfig`
Configuration for the streaming loop.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| samples_per_packet | usize | 25 | Number of samples in one UDP packet |
| speed_multiplier | f64 | 1.0 | Speed factor (0.5 = slow, 2.0 = fast) |

### `PacketChunk`
A slice of data ready for transmission.

| Field | Type | Description |
|-------|------|-------------|
| samples | Vec<i32> | Subset of record samples |
| start_time | f64 | Exact start time of this chunk (Unix float) |
| duration | f64 | Expected duration in seconds |

## Process Flow

1. **Parse Record**: Decode MiniSEED record -> `Segment`.
2. **Chunking**: Split `Segment.samples` into `PacketChunk`s of size 25.
3. **Transmission**:
   - For each chunk:
     - Serialize to `rsudp` format string.
     - Send UDP.
     - Wait `chunk.duration / speed_multiplier`.
