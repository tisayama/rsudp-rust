# Research: UDP Data Forwarding (034-udp-forward)

**Date**: 2026-02-10
**Branch**: `034-udp-forward`

## Decision 1: Forwarding Tap Point

**Decision**: Forward raw `Vec<u8>` bytes from within `run_pipeline()`, after `parse_any()` determines the channel name but using the original raw byte buffer for the actual send.

**Rationale**:
- The Python rsudp `c_forward.py` forwards raw packet bytes via `sock.sendto(p, (self.addr, self.port))` — no re-serialization.
- Channel filtering requires knowing the channel name, which is only available after parsing (`TraceSegment.channel`).
- The pipeline already receives raw bytes (`mpsc::Receiver<Vec<u8>>`) and parses them. We keep a reference to the raw bytes, parse to get channel info, then conditionally forward the original bytes.
- This preserves exact wire format compatibility with downstream receivers expecting Raspberry Shake protocol packets.

**Alternatives considered**:
- **Receiver-level forwarding (before pipeline)**: Would forward all packets without channel filtering. Rejected because filtering is a core requirement (FR-003).
- **Post-parse re-serialization**: Would serialize `TraceSegment` back to bytes. Rejected because it changes the wire format and adds unnecessary overhead.

## Decision 2: Forwarding Architecture Pattern

**Decision**: Create a dedicated `Forward` struct with a `tokio::spawn`-ed async task per destination. Each task owns a `UdpSocket` and receives forwarding commands via a `tokio::sync::mpsc` channel.

**Rationale**:
- Matches the Python pattern where each destination has its own thread.
- Isolates forwarding I/O from the main pipeline processing loop — a slow or unreachable destination cannot block parsing/triggering.
- Uses the existing `tokio::spawn` consumer pattern established by Hue, Audio, and SNS consumers.
- Each task can maintain its own stats counters for periodic logging (FR-008).

**Alternatives considered**:
- **Inline in pipeline loop**: Simpler but `sendto()` on an unreachable destination could block or slow the pipeline. Rejected per FR-007.
- **Single shared task for all destinations**: Simpler but a single slow destination could delay forwarding to all others. Rejected for isolation.

## Decision 3: ALARM/RESET Message Format for Forwarding

**Decision**: Format ALARM and RESET messages as UTF-8 strings matching the Python rsudp format: `ALARM {channel} {timestamp}` and `RESET {channel} {timestamp}`.

**Rationale**:
- The Python `c_forward.py` checks for the literal strings `'ALARM'` and `'RESET'` in the packet bytes (line: `if 'ALARM' in p.decode('utf-8')`).
- Downstream receivers expecting rsudp-compatible messages need this exact format.
- Simple, human-readable, and easy to filter/detect.

**Alternatives considered**:
- **JSON format**: More structured but incompatible with Python rsudp receivers. Rejected for compatibility.
- **No alarm forwarding**: User Story 2 and FR-005 explicitly require alarm forwarding support. Rejected.

## Decision 4: Channel Matching Strategy

**Decision**: Case-insensitive suffix matching. A filter channel "HZ" matches any channel ending with "HZ" (e.g., "EHZ", "SHZ", "BHZ"). The special value "all" matches all channels.

**Rationale**:
- The Python implementation converts both the filter and available channels to uppercase and uses `in` comparison (line: `if cha[i].upper() in c.upper()`).
- This effectively performs substring matching, which in practice acts as suffix matching for standard SEED channel codes.
- Matches the existing `TriggerConfig.target_channel` pattern in `trigger.rs` which also uses suffix matching on "HZ".

**Alternatives considered**:
- **Exact match only**: Would require users to specify full channel codes (e.g., "EHZ" instead of "HZ"). Less flexible.
- **Regex matching**: Over-engineered for this use case.

## Decision 5: Stats Logging Interval

**Decision**: Log forwarding statistics every 60 seconds during active forwarding.

**Rationale**:
- Frequent enough to detect issues quickly (SC-005 requires status within 30 seconds of startup — initial startup log handles this).
- Infrequent enough to avoid log noise at typical Raspberry Shake data rates (~100 samples/sec).
- Aligns with the `IntensityManager` 60-second window pattern already in the codebase.

**Alternatives considered**:
- **10 seconds**: Too frequent, excessive log output for a monitoring system running 24/7.
- **300 seconds (5 min)**: Too infrequent for detecting forwarding failures promptly.

## Decision 6: Queue Overflow Strategy

**Decision**: Use a bounded `mpsc::channel` (capacity 32) per forwarding task. If the channel is full, drop the packet and increment a `dropped` counter logged in periodic stats.

**Rationale**:
- FR-007 requires non-blocking behavior. A bounded channel with `try_send()` guarantees the pipeline never waits.
- Capacity 32 provides ~0.3 seconds of buffer at 100 packets/sec, enough to absorb brief send latency spikes.
- The dropped counter provides visibility into backpressure (FR-008).

**Alternatives considered**:
- **Unbounded channel**: Could consume unlimited memory if a destination is permanently down. Rejected.
- **No queue (inline send)**: Blocks pipeline on each send. Rejected per FR-007.
