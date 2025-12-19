# Research: STA/LTA Alert System

## Decision: STA/LTA Algorithm Parity
- **Decision**: Use the recursive STA/LTA formula already implemented in `src/trigger.rs` (Feature 003).
- **Rationale**: It matches ObsPy's C implementation (`recstalta.c`) exactly and is already verified with a Python cross-validation script.
- **Parity Note**: The implementation correctly zeros out the first `nlta` samples and handles the initial count offset.

## Decision: Digital Filtering Implementation
- **Decision**: Implement a stateful IIR filter (Direct Form II Transposed or Direct Form I) using Biquads. For filter design (coefficient generation), evaluate using the `butterworth` crate or implementing the Bilinear Transform.
- **Rationale**: `rsudp` uses ObsPy's `filter` method which is a standard Butterworth filter. For real-time processing in Rust, a stateful filter is more efficient than the "slice and filter" approach used in the Python version, while remaining mathematically equivalent if state is preserved.
- **Alternative**: Stick to "slice and filter" to minimize state complexity, but this would be less performant and potentially introduce artifacts at window boundaries. Stateful is preferred.

## Decision: Data Gap Handling
- **Decision**: Detect gaps by checking the difference between the current sample timestamp and the expected timestamp (based on sample rate).
- **Rationale**: If a gap is detected, the recursive averages (STA/LTA) and filter state must be reset to avoid artifacts from discontinuous data.

## Decision: Event Notification System
- **Decision**: Use `tokio::sync::broadcast` or `tokio::sync::mpsc` for emitting internal `AlertEvent` messages.
- **Rationale**: This allows other components (like logging or potential future network consumers) to react to alarms without coupling the alert logic to specific output handlers.
