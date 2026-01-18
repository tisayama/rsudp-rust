# Research: Packet Granularity Optimization

## Decisions

### Decision: Default 25 Samples per Packet
**Rationale**: `rsudp` documentation and test data examples consistently show 25 samples per packet for 100Hz data. This corresponds to a 0.25s update interval, which is likely the optimal refresh rate for its plotting library.
**Alternatives considered**:
- **Variable based on MTU**: Too complex and `rsudp` seems to expect specific timing.
- **1 sample per packet**: Too much overhead.

### Decision: Transmission Loop Logic
**Rationale**:
1. Read a MiniSEED record (e.g., 500 samples).
2. Calculate chunk count: `500 / 25 = 20` chunks.
3. Loop 20 times:
   - Extract samples `i*25` to `(i+1)*25`.
   - Calculate chunk timestamp: `RecordStart + (i * 25 / SampleRate)`.
   - Format and send packet.
   - Sleep for `25 / SampleRate` (0.25s).
**Constraint**: Sleep must account for processing time to avoid drift, but simple `tokio::time::sleep` is likely sufficient for simulation purposes given `rsudp`'s loose tolerance.

## Research Tasks

### Task: Validate Sample Rate Dependency
**Decision**: Assume 100Hz for default calculation, but use `segment.sample_rate` from the MiniSEED header dynamically.
**Formula**: `duration = chunk_size / sample_rate`
