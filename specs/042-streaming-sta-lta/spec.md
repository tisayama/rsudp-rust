# Feature Specification: Streaming STA/LTA Trigger Calculation

**Feature Branch**: `042-streaming-sta-lta`
**Created**: 2026-03-02
**Status**: Draft
**Input**: User description: "STA/LTA トリガー計算をスライス再計算方式からストリーミング方式に変更する"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Accurate Earthquake Alert Reset Timing (Priority: P1)

A seismic monitoring operator running rsudp-rust on a Raspberry Pi receives earthquake alerts (ALARM) and expects the system to reset (RESET) within a timeframe consistent with the original rsudp Python implementation. Currently, RESET takes 159 seconds instead of the expected 72 seconds for the same seismic event, causing prolonged false-alarm states where Hue lights remain flashing, alert sounds cannot re-trigger, and the operator incorrectly perceives the system as stuck.

**Why this priority**: This is the core defect. The 2.2x delay in RESET timing directly impacts operational reliability and user trust in the alerting system.

**Independent Test**: Can be verified by streaming `references/mseed/shindo0.mseed` through the system and measuring ALARM-to-RESET duration. A correct implementation produces ~72 seconds (±5s), matching the rsudp Python reference.

**Acceptance Scenarios**:

1. **Given** a seismic event in `shindo0.mseed`, **When** the data is streamed through the trigger system with STA=6s, LTA=30s, threshold=1.1, reset=0.5, highpass=0.1, lowpass=2.0, **Then** the ALARM-to-RESET duration is 72 seconds ±5 seconds.
2. **Given** the same seismic event, **When** comparing max STA/LTA ratio, **Then** the peak ratio matches the rsudp Python reference (approximately 4.5) rather than the current degraded value (~3.9).
3. **Given** the system has been running for 10+ minutes with quiet data, **When** a seismic event occurs, **Then** the ALARM triggers at the same time (±1 second) as the rsudp Python reference.

---

### User Story 2 - Continuous Operation Without State Corruption (Priority: P2)

An operator deploys rsudp-rust as a long-running service (days/weeks). The trigger system must process seismic data continuously without accumulating calculation errors, memory growth, or degraded detection accuracy over time.

**Why this priority**: Streaming state persistence introduces the risk of numerical drift or state corruption over extended operation periods.

**Independent Test**: Can be verified by running the system for an extended period with synthetic data and confirming that trigger behavior remains consistent.

**Acceptance Scenarios**:

1. **Given** the system has been running for 24+ hours processing continuous data, **When** a seismic event occurs, **Then** the ALARM/RESET timing is identical to the timing observed at startup.
2. **Given** a 1-second gap in the data stream (e.g., network dropout), **When** data resumes, **Then** the filter and STA/LTA state reset cleanly and the system recovers within LTA seconds (30s) of warmup.
3. **Given** the system is processing data at 100 Hz, **When** memory usage is monitored over 24 hours, **Then** there is no memory growth attributable to the trigger calculation (fixed-size state, no growing buffers).

---

### User Story 3 - Preserved Alert Behavior and Integration (Priority: P3)

All existing alert features — duration-based triggering, threshold/reset logic, periodic status reports, and downstream consumers (Hue lights, audio playback, capture screenshots, messaging services) — continue to function identically after the internal calculation method changes.

**Why this priority**: This is a refactoring change; no external behavior should change except for the corrected timing.

**Independent Test**: Can be verified by running the existing E2E test suite and confirming all trigger events (ALARM, RESET, STATUS) are emitted with correct data.

**Acceptance Scenarios**:

1. **Given** the trigger threshold is set to 1.1 and duration to 0.0, **When** the STA/LTA ratio exceeds 1.1, **Then** an ALARM event is emitted immediately (no regression from current behavior).
2. **Given** the trigger duration is set to 2.0 seconds, **When** the STA/LTA ratio exceeds the threshold, **Then** the ALARM is only emitted after the ratio has remained above the threshold for 2.0 continuous seconds.
3. **Given** the system is in triggered state, **When** periodic status reports are requested, **Then** STATUS events are emitted with the current ratio and max_ratio values.

---

### Edge Cases

- What happens when the very first sample arrives? The system must handle initialization gracefully (STA=0, LTA=near-zero, ratio=0).
- What happens when a data gap of exactly 1 second occurs? The threshold is ">1 second" gap, so exactly 1 second should NOT reset state.
- What happens when a data gap of 1.001 seconds occurs? State should reset.
- What happens when multiple channels share the same target pattern (e.g., both "EHZ" channels)? Each channel must maintain independent streaming state.
- What happens when the sample rate changes mid-stream? The system uses a fixed 100 Hz assumption; behavior is undefined for other rates (existing limitation, not addressed by this change).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST maintain bandpass filter state (Biquad s1/s2 coefficients) persistently across samples for each channel, rather than reinitializing to zero on each sample.
- **FR-002**: The system MUST maintain STA and LTA values persistently across samples for each channel, updating them incrementally with each new filtered sample using the recursive formula: `sta = (1/nsta) * energy + (1 - 1/nsta) * sta` and `lta = (1/nlta) * energy + (1 - 1/nlta) * lta`.
- **FR-003**: The system MUST initialize filter coefficients once at startup (or on first sample for each channel), not on every sample.
- **FR-004**: The system MUST suppress ratio output (return 0.0) until at least `nlta` samples (LTA window length) have been processed, to allow the LTA to warm up.
- **FR-005**: The system MUST count processed samples per channel to track warmup progress.
- **FR-006**: When a data gap greater than 1 second is detected between consecutive samples, the system MUST reset filter state (s1=s2=0 for all Biquad sections), STA, LTA, and sample counter to their initial values.
- **FR-007**: The system MUST NOT use a sample buffer (VecDeque or similar) for STA/LTA calculation; all state must be carried in fixed-size per-channel structures.
- **FR-008**: The system MUST preserve all existing trigger logic: threshold comparison, reset comparison, duration-based triggering, max_ratio tracking, and periodic STATUS event emission.
- **FR-009**: The system MUST produce ALARM-to-RESET timing within ±5 seconds of the rsudp Python reference implementation when processing identical input data.

### Key Entities

- **StaLtaState**: Per-channel streaming state containing: filter sections (Biquad array with persistent s1/s2), current STA value, current LTA value, sample counter, triggered flag, max_ratio, timing fields for duration-based triggering, and last timestamp for gap detection.
- **TriggerManager**: Manages per-channel StaLtaState instances and trigger configuration. Processes one sample at a time in O(1) per sample (no buffer iteration).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: ALARM-to-RESET duration for `shindo0.mseed` is 72 seconds ±5 seconds (currently 159.5 seconds).
- **SC-002**: Peak STA/LTA ratio for `shindo0.mseed` is approximately 4.5 (currently 3.9), matching the rsudp Python reference.
- **SC-003**: Processing cost per sample is O(1) — no per-sample buffer iteration or filter reinitialization (currently O(3100) per sample).
- **SC-004**: Memory usage per channel is fixed (no growing buffers); the VecDeque buffer is eliminated entirely.
- **SC-005**: All existing trigger events (ALARM, RESET, STATUS) continue to be emitted with correct payloads; no downstream integration regressions.
- **SC-006**: System operates continuously for 24+ hours without numerical drift or state corruption affecting trigger accuracy.

## Assumptions

- Sample rate is assumed to be 100 Hz (hardcoded as in current implementation). Dynamic sample rate support is out of scope.
- The existing hardcoded Butterworth bandpass filter coefficients (4th order, 0.1–2.0 Hz at 100 Hz) are correct and will be reused as-is.
- The rsudp Python/ObsPy `recursive_sta_lta` implementation is the authoritative reference for expected behavior.
- The verification scripts `verify_stalta.py` and `verify_rust_stalta.py` already exist and can be adapted to validate the fix.
- This change is internal to `trigger.rs`; the `add_sample()` public API signature remains unchanged.

## Scope Boundaries

### In Scope
- Refactoring `StaLtaState` to use streaming (persistent) filter and STA/LTA state
- Removing the VecDeque sample buffer
- Adapting gap detection to reset streaming state instead of clearing a buffer
- Verification against rsudp Python reference using `shindo0.mseed`

### Out of Scope
- Dynamic filter coefficient calculation (replacing hardcoded coefficients)
- Dynamic sample rate detection
- Changes to the `TriggerConfig` or `AlertEvent` public APIs
- Changes to downstream consumers (pipeline, Hue, audio, capture, messaging)
- Performance optimization beyond the inherent O(1) improvement from eliminating buffer iteration
