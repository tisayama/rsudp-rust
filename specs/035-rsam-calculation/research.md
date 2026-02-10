# Research: RSAM Calculation and UDP Forwarding

**Feature**: 035-rsam-calculation
**Date**: 2026-02-10

## Decision 1: RSAM Integration Architecture

**Decision**: Follow the Forward module pattern — RsamManager struct with async task for UDP sending, fed from pipeline via method call on each matching segment.

**Rationale**: The Forward module (added in 034-udp-forward) is the most recent consumer added to the pipeline and establishes a proven pattern. It uses `Option<Arc<ForwardManager>>` passed to `run_pipeline()`, with non-blocking method calls in the processing loop. RSAM has similar requirements: receive segments, accumulate data, periodically send UDP.

**Alternatives considered**:
- Separate tokio task reading from dedicated mpsc channel: Adds unnecessary complexity since RSAM only needs sample accumulation, not per-packet forwarding.
- Inline calculation in pipeline.rs: Would bloat the already-long pipeline function. Separate module is cleaner.

## Decision 2: Sample Buffer Strategy

**Decision**: Use a simple `Vec<f64>` with time-based eviction. On each segment arrival, append absolute sample values. On interval tick, calculate statistics from the buffer and clear it.

**Rationale**: RSAM uses a fixed time window (default 10 seconds). At 100 Hz sampling rate, this is ~1000 samples — trivially small. A `Vec<f64>` with periodic clear is simpler and faster than a ring buffer for this use case. The Python implementation similarly accumulates then slices.

**Alternatives considered**:
- Ring buffer (VecDeque): Overhead of maintaining head/tail pointers not justified for ~1000 elements cleared every 10 seconds.
- Timestamp-indexed buffer: More precise but unnecessary since we clear on interval boundaries.

## Decision 3: Interval Timing Mechanism

**Decision**: Use `tokio::time::Instant` tracking within the `process_segment()` method. When elapsed time exceeds the configured interval, calculate and send, then reset the timer.

**Rationale**: RSAM runs within the pipeline's processing loop, not as a separate async task. Tracking elapsed time with `Instant::elapsed()` is simple and avoids the complexity of a separate timer task with channel communication. The Python implementation uses `time.time() > next_int` — same approach.

**Alternatives considered**:
- Separate `tokio::time::interval` task with mpsc channel: Would require spawning a task and passing samples through a channel, similar to Forward. Overkill since RSAM has a single destination and doesn't need non-blocking send (UDP send is fast).
- Timer in pipeline loop: Would couple RSAM timing to pipeline, which is undesirable.

## Decision 4: Deconvolution Implementation

**Decision**: Apply sensitivity conversion inline in `process_segment()`. When `deconvolve=true`, divide each sample by the channel's sensitivity value before taking absolute value. Support VEL, ACC, DISP, GRAV, CHAN modes.

**Rationale**: The sensitivity map (`HashMap<String, f64>`) is already available in the pipeline and fetched at startup from FDSN StationXML. The conversion is straightforward: `sample / sensitivity`. For GRAV mode, additionally divide by 9.81. For CHAN mode, use VEL for EH* channels and ACC for EN* channels (matching Python rsudp behavior).

**Alternatives considered**:
- Pre-converting all pipeline samples: Would affect other consumers. RSAM should convert locally.
- Ignoring deconvolution: User explicitly requested it.

## Decision 5: Output Format Implementation

**Decision**: Implement formatting as simple `format!()` string operations. Three formats: LITE (pipe-delimited), JSON (manual string format), CSV (comma-separated).

**Rationale**: The output is a single line per interval. Using `serde_json` for JSON formatting would add unnecessary dependency usage for such a simple structure. Manual `format!()` is more explicit and matches the Python implementation's approach.

**Alternatives considered**:
- `serde_json::to_string()`: Would require making RsamResult derive Serialize. Overkill for 6 fields.
- Custom trait for formatting: Over-engineering for 3 simple formats.

## Decision 6: Channel Matching Reuse

**Decision**: Reuse the existing `forward::should_forward_channel()` function for RSAM channel matching.

**Rationale**: RSAM uses the same suffix-matching logic as Forward (e.g., "HZ" matches "EHZ"). The function is already `pub` and handles case-insensitive matching, "all" wildcard, and empty filter fallback. No need to duplicate.

**Alternatives considered**:
- Duplicate the function in rsam.rs: Code duplication.
- Extract to a shared utility module: Premature abstraction for a single function used by two modules.

## Decision 7: UDP Socket Management

**Decision**: Bind a single UDP socket at RsamManager initialization. Send directly in the processing method (non-blocking `send_to` via std::net::UdpSocket since sends are infrequent).

**Rationale**: RSAM sends one packet every N seconds (default 10). Unlike Forward which sends per-packet and needs async sockets, RSAM can use a blocking `std::net::UdpSocket::send_to()` since it's called once per interval and UDP sends are essentially instant for small payloads (~100 bytes).

**Alternatives considered**:
- Async tokio UdpSocket with spawned task: Overkill for 1 send every 10 seconds.
- Share socket with Forward module: Unnecessary coupling.

## Decision 8: Station Name in Output

**Decision**: Use the station name from the first matching TraceSegment (e.g., "R6E01"). Not the full NSLC (network.station.location.channel).

**Rationale**: Python rsudp uses just the station name in RSAM output (e.g., `stn:R3BCF`). For compatibility with existing receivers expecting Python rsudp format, we should match this behavior.

**Alternatives considered**:
- Full NSLC: Too verbose, breaks compatibility with existing receivers.
- Network.Station: Not what Python rsudp sends.
