# Data Model: Google Cloud Pub/Sub Integration

**Feature**: 040-pubsub-integration | **Date**: 2026-02-13

## Entity Definitions

### 1. SeismicBatch (Pub/Sub Message Payload)

A batch message aggregating seismic data from all channels within a 0.5-second window. Serialized with Protocol Buffers and stored in the Pub/Sub message `data` field.

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| station | string | Station identifier (e.g., `AM.R6E01`) | Non-empty, `{net}.{sta}` format |
| window_start_ms | int64 | Window start time (Unix milliseconds) | > 0, floored to 500ms boundary |
| window_end_ms | int64 | Window end time (Unix milliseconds) | > window_start_ms |
| sample_rate | double | Sample rate (Hz) | > 0 (typically 100.0) |
| channels | repeated ChannelData | Per-channel sample data | At least 1 entry |

### 2. ChannelData

Sample data for a single channel within a 0.5-second window.

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| channel | string | Channel name (e.g., `EHZ`, `ENE`) | Non-empty |
| samples | repeated int32 | Raw integer counts (pre-sensitivity) | packed encoding |
| start_time_ms | int64 | Channel data start time (Unix ms) | > 0 |

### 3. Pub/Sub Message Attributes

Message-level metadata stored outside the payload, used for filtering and deduplication.

| Attribute Key | Type | Description | Example |
|--------------|------|-------------|---------|
| dedup_key | string | Deterministic deduplication key | `AM.R6E01:2025-11-25T09:01:23.500Z` |
| station | string | Station identifier | `AM.R6E01` |
| window_start | string | Window start time (ISO 8601) | `2025-11-25T09:01:23.500Z` |

### 4. Pub/Sub Ordering Key

| Key | Format | Description |
|-----|--------|-------------|
| ordering_key | `{network}.{station}` | Per-station ordering guarantee |

## State Transitions

### Publisher Buffer State

```
Empty → Accumulating → Ready → Publishing → Empty
                         ↑         |
                         └─────────┘ (retry on failure)
```

- **Empty**: 0.5-second window starts, buffer empty
- **Accumulating**: Channel data added to buffer on each UDP packet arrival
- **Ready**: 0.5 seconds elapsed, batch message constructed
- **Publishing**: Publishing to Pub/Sub in progress
- **Empty**: After success, advance to next window

### Subscriber Deduplication State

```
Received → Check dedup_key → [new] → Process → Ack
                             [dup] → Skip → Ack
```

- Managed via dedup_key HashSet (max 10,000 entries, LRU purge)

## Configuration Entity

### PubsubSettings (rsudp.toml `[pubsub]` section)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| enabled | bool | false | Enable/disable Pub/Sub publisher |
| project_id | String | "" | Google Cloud project ID |
| topic | String | "" | Pub/Sub topic name |
| subscription | String | "" | Subscription name for subscriber mode |
| credentials_file | Option\<String\> | None | Service account JSON file path |
| input_mode | String | "udp" | Input source: "udp" or "pubsub" |
| batch_interval_ms | u64 | 500 | Batch aggregation window (milliseconds) |

### Credential Resolution Priority

1. `PUBSUB_EMULATOR_HOST` environment variable is set → Connect to emulator (no auth required)
2. `credentials_file` is configured → Use JSON at that path
3. `GOOGLE_APPLICATION_CREDENTIALS` environment variable → Use JSON at that path
4. None of the above → Log error, disable Pub/Sub functionality

## Relationships

```
Settings
└── PubsubSettings
    ├── → PubsubPublisher (initialized when enabled=true and input_mode="udp")
    │     └── uses → SeismicBatch (protobuf encode)
    │         └── contains → ChannelData[]
    └── → PubsubSubscriber (initialized when enabled=true and input_mode="pubsub")
          └── uses → SeismicBatch (protobuf decode)
              └── converts to → rsudp Packet format → pipe_tx
```

## Data Flow

### Publish Path

```
UDP Packet → parse_any() → Segment
    → PubsubPublisher.buffer_segment(segment)
    → [0.5 seconds elapsed]
    → SeismicBatch { channels: [buffered_channels] }
    → prost::Message::encode_to_vec()
    → PubsubMessage { data, attributes: {dedup_key}, ordering_key }
    → publisher.publish(msg)
```

### Subscribe Path

```
subscription.receive(message)
    → Check dedup_key against HashSet
    → [new] SeismicBatch::decode(message.data)
    → For each ChannelData in batch:
        → Reconstruct rsudp JSON packet format
        → pipe_tx.send(packet_bytes)
    → message.ack()
```
