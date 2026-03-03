# Feature Specification: Classic STA/LTA Algorithm

**Feature Branch**: `043-classic-stalta`
**Created**: 2026-03-02
**Status**: Draft
**Input**: User description: "STA/LTAアルゴリズムをRecursive (EMA) からClassic (スライディングウィンドウ) に変更し、通常ノイズでの誤報を防止する"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - False Alarm Elimination During Normal Conditions (Priority: P1)

As a seismic monitoring operator, I need the system to produce zero false earthquake alerts during normal ambient noise conditions, so that I can trust the alert system and avoid unnecessary alarm fatigue from continuous false notifications.

**Why this priority**: False alarms during normal operation undermine trust in the system. The current implementation produces 17-21 false alarms per hour during quiet conditions, making the system unusable for practical monitoring.

**Independent Test**: Stream 60 minutes of normal (non-earthquake) seismic data through the trigger system and verify that zero ALARM events are produced.

**Acceptance Scenarios**:

1. **Given** a stream of normal ambient seismic data (no earthquake) with a duration of 60 minutes, **When** the data is processed through the STA/LTA trigger system with threshold=1.1 and reset=0.5, **Then** zero ALARM events are generated.
2. **Given** a second independent normal ambient dataset of 60 minutes, **When** processed with the same configuration, **Then** zero ALARM events are generated.
3. **Given** normal ambient data, **When** the STA/LTA ratio is computed continuously, **Then** the ratio remains below the trigger threshold (1.1) at all times during the observation period.

---

### User Story 2 - Earthquake Detection Performance Preservation (Priority: P1)

As a seismic monitoring operator, I need the system to detect real earthquakes with the same timing accuracy as the reference implementation, so that alerts arrive promptly and reset at the correct time.

**Why this priority**: Detection accuracy is equally critical as false alarm suppression. If the algorithm change degrades earthquake detection, the system fails its primary purpose.

**Independent Test**: Stream reference earthquake data (shindo-0 class event) through the trigger system and verify ALARM-to-RESET duration matches the reference implementation within acceptable tolerance.

**Acceptance Scenarios**:

1. **Given** seismic data containing a shindo-0 earthquake event, **When** processed through the trigger system, **Then** exactly one ALARM event is generated.
2. **Given** the same earthquake data, **When** the ALARM-to-RESET duration is measured, **Then** it falls within 72 seconds +/- 5 seconds.
3. **Given** the same earthquake data, **When** the maximum STA/LTA ratio during the alert period is measured, **Then** it is sufficient to clearly distinguish the earthquake from background noise (ratio significantly above threshold).

---

### User Story 3 - Continuous Operation Stability (Priority: P2)

As a system administrator, I need the trigger system to operate continuously for extended periods (days to weeks) without numerical drift or memory growth, so that the system can run unattended.

**Why this priority**: A monitoring system that degrades over time requires manual restarts, reducing reliability and operator confidence.

**Independent Test**: Run the system continuously for an extended period and verify that computation accuracy and memory usage remain stable.

**Acceptance Scenarios**:

1. **Given** the system has been running continuously for 24 hours, **When** the STA/LTA computation accuracy is checked, **Then** the computed values have no measurable numerical drift compared to a fresh calculation on the same data window.
2. **Given** the system encounters a data gap exceeding 1 second, **When** data resumes, **Then** the system resets all internal state and re-enters a warmup period before resuming alert evaluation.
3. **Given** the system has been running for 24 hours, **When** memory usage is measured, **Then** memory consumption has not grown beyond the initial allocation (bounded memory).

---

### Edge Cases

- What happens when a data gap of exactly 1 second occurs? (Boundary: gaps > 1 second trigger reset, gaps <= 1 second do not.)
- How does the system behave during the warmup period if an earthquake occurs? (Expected: no alert during warmup, ratio is suppressed to 0.0.)
- What happens when the incoming data has constant zero amplitude? (Expected: no division-by-zero errors, ratio remains 0.0 or safely handled.)
- How does the system handle a sudden change in background noise level (e.g., construction noise starting)? (Expected: LTA adapts within the LTA window duration, no persistent false alarms.)
- What happens at the boundary between warmup completion and normal operation? (Expected: smooth transition, no spurious trigger at the moment warmup ends.)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST compute STA as the arithmetic mean of the most recent STA-window-length energy values (energy = filtered sample squared).
- **FR-002**: The system MUST compute LTA as the arithmetic mean of the most recent LTA-window-length energy values.
- **FR-003**: The system MUST compute the STA/LTA ratio as STA divided by LTA for each incoming sample.
- **FR-004**: The system MUST use bounded-memory storage for energy values, retaining only the most recent LTA-window-length values.
- **FR-005**: The system MUST update the STA and LTA values in constant time (O(1)) per incoming sample using running sums with addition and subtraction.
- **FR-006**: The system MUST periodically recalculate running sums from scratch to prevent floating-point accumulation errors from degrading accuracy over extended operation.
- **FR-007**: The system MUST maintain persistent bandpass filter state across samples (no re-initialization per sample or per packet).
- **FR-008**: The system MUST suppress the STA/LTA ratio to 0.0 during the warmup period (until at least LTA-window-length samples have been accumulated).
- **FR-009**: The system MUST reset all internal state (filter state, energy buffer, running sums, sample counter) when a data gap exceeding 1 second is detected, returning to warmup mode.
- **FR-010**: The system MUST preserve all existing trigger logic unchanged: threshold-based alarm activation, reset-threshold-based deactivation, duration-based debounce, and periodic status reporting.
- **FR-011**: The system MUST keep total memory usage for the energy buffer within 24 KB (3000 values at 8 bytes each for the default 30-second LTA at 100 samples per second).

### Key Entities

- **Energy Buffer**: A fixed-size circular buffer storing squared filtered sample values. Its capacity equals the LTA window length in samples. Used to compute both STA and LTA running sums.
- **Running Sums**: Two cumulative sum values (one for STA window, one for LTA window) that are incrementally updated as new energy values enter the buffer and old values exit. Subject to periodic full recalculation for numerical stability.
- **Filter State**: Persistent bandpass filter coefficients and internal state (s1, s2 per biquad section) that process each incoming raw sample before energy computation. Preserved across samples; reset only on data gaps.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero false ALARM events are produced when processing 60 minutes of normal ambient seismic noise data (verified against two independent reference datasets).
- **SC-002**: Earthquake alert timing (ALARM-to-RESET duration) falls within 72 seconds +/- 5 seconds when processing reference earthquake data, matching the reference implementation.
- **SC-003**: Each incoming sample is processed in constant time, with no per-sample computation scaling with buffer size or history length.
- **SC-004**: Total memory usage for the energy history buffer does not exceed 24 KB under default configuration (30-second LTA at 100 samples per second).
- **SC-005**: After 24 hours of continuous operation, computed STA/LTA values show no measurable numerical drift compared to a fresh recalculation on the same data window.
- **SC-006**: All existing alert behaviors (trigger, reset, status events) continue to function identically to the previous version for real earthquake events.

## Assumptions

- The sampling rate is 100 samples per second (100 SPS), consistent with the existing system and reference data.
- The default STA window is 6 seconds (600 samples) and the default LTA window is 30 seconds (3000 samples).
- The default trigger threshold is 1.1 and the default reset threshold is 0.5, as configured in the current deployment.
- The bandpass filter configuration (0.1-2.0 Hz, 4th order Butterworth) remains unchanged.
- Reference test data (shindo0.mseed, normal.mseed, normal2.mseed) is available for validation.
- The periodic recalculation interval for running sums is an implementation detail; the specification requires only that drift be prevented over 24+ hours of operation.
