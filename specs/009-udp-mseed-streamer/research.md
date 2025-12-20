# Research: UDP MiniSEED Streamer for Simulation

## Decisions & Rationale

### 1. Timing Precision
- **Decision**: Use `tokio::time::sleep` combined with a high-resolution base clock for interval calculation.
- **Rationale**: While standard `std::thread::sleep` is blocked by the OS scheduler, `tokio::time::sleep` integrated with the async runtime provides sufficient precision for the 5ms jitter requirement, especially when supplemented with a "drift correction" logic that calculates the next wake-up time based on absolute start time rather than relative delays.
- **Alternatives considered**: Busy-waiting (too high CPU usage), `timerfd` on Linux (unnecessary complexity).

### 2. Large File Indexing
- **Decision**: Pre-scan the MiniSEED file to create an in-memory index of `(StartTime, FileOffset, Length, ChannelID)`.
- **Rationale**: Allows 2GB+ files to be processed with <10MB memory usage. Sorting occurs on small index entries rather than raw data. On-demand `seek` and `read` during playback keep the memory footprint constant regardless of file size.
- **Alternatives considered**: Full file load (fails SC-002), streaming sort (impossible for non-ordered files).

### 3. UDP Packet Format (rsudp compatible)
- **Decision**: Implement the specific string-based JSON-like format: `{'CHAN', timestamp, sample1, sample2, ...}`.
- **Rationale**: Matches the user requirement for `rsudp` format compatibility (Clarification Session 2025-12-19). This allows the streamer to act as a drop-in replacement for a Raspberry Shake device sending data to any `rsudp`-compatible receiver.
- **Alternatives considered**: Raw binary forwarding (Rejected in clarification).

### 4. Concurrency Model
- **Decision**: Single-threaded async loop for streaming to ensure strict temporal order.
- **Rationale**: Since the requirement is strict time-sequence playback across all channels (Clarification Session 2025-12-19), a single sorted queue processed by a single loop is more reliable than per-channel threads which might drift relative to each other.

## Best Practices Found

### Rust UDP Performance
- Use `std::net::UdpSocket` wrapped in `tokio::net::UdpSocket`.
- Set `SO_SNDBUF` if high-speed bursts are expected (though not typical for seismic data).

### High-Resolution Sleep in Rust
- For jitter < 5ms, drift correction is mandatory: `let next_tick = start_instant + Duration::from_micros(total_micros_to_wait);`.
