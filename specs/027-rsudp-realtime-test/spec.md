# Feature Specification: RSUDP Realtime Test Run

**Feature Branch**: `027-rsudp-realtime-test`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "fdsws.mseedでは少なくともアラートが1回でますよ。100倍速でうまくいかない可能性もありますので、1倍速でやってほしいです。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Realtime Playback Comparison (Priority: P1)

As a developer, I want to run the comparison test at 1x speed (realtime) to ensure that timing-dependent logic (like STA/LTA warm-up and LTA window filling) functions correctly and detects the expected event in `fdsnws.mseed`.

**Why this priority**: Fast-forwarding playback can cause race conditions or buffer issues in real-time processing systems. Verifying at 1x speed removes this variable.

**Independent Test**:
1. Modify `run_python_ref.sh` and `run_rust_target.sh` to use `--speed 1.0`.
2. Execute `run_comparison.sh` (this will take ~20-30 minutes).
3. Verify that both implementations detect at least one trigger event.
4. Compare the trigger times in `comparison_report.csv`.

**Acceptance Scenarios**:

1. **Given** `fdsnws.mseed` played at 1x speed, **When** processed by both systems, **Then** both MUST log a trigger event.
2. **Given** the trigger events, **When** compared, **Then** the timestamps MUST be within ±0.5 seconds of each other.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The test runner scripts MUST execute `streamer` with `--speed 1.0`.
- **FR-002**: The scripts MUST wait for the full duration of the stream (approx. 10-15 minutes) without timing out or killing the process prematurely.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Comparison report shows at least 1 matched event between Python and Rust.
- **SC-002**: Time difference between matched events is < 0.5 seconds.