# Tasks: Restore STA/LTA Alerts Functionality

**Feature**: Restore STA/LTA Alerts Functionality
**Status**: Completed
**Implementation Strategy**: Diagnose filter state persistence with unit tests, then fix `TriggerManager` to handle arbitrary chunk sizes.

## Phase 1: Setup
Goal: Reproduce the issue with a targeted test.

- [X] T001 [US1] Create a reproduction unit test in `rsudp-rust/src/trigger.rs` that feeds data in varying chunk sizes (e.g., 1 vs 100 samples) and asserts identical STA/LTA output
- [X] T002 Verify `rsudp-rust/src/trigger.rs` build status

## Phase 2: Foundational
Goal: Fix the underlying filter logic.

- [X] T003 [US1] Inspect `BandpassFilter::process` in `rsudp-rust/src/trigger.rs` and ensure `x1`, `x2`, `y1`, `y2` are updated correctly per sample, not per batch call
- [X] T004 [US1] Modify `BandpassFilter` or `TriggerManager` state management if T001 fails, ensuring state is persistent across `add_sample` calls
- [X] T005 [P] [US1] Verify `TriggerManager::add_sample` logic doesn't reset internal counters (`sample_count`) prematurely

## Phase 3: [US1] Restore Alert Triggering (Priority: P1)
Goal: Ensure alerts fire with the fixed logic.

- [X] T006 [US1] Run the `test_sta_lta_trigger` integration test with simulated small chunks (25 samples) to verify fix
- [X] T007 [US1] Verify that `pipeline.rs` correctly passes timestamps (already fixed in previous feature, but double-check integration)

## Phase 4: Integration & Verification
Goal: End-to-end verification.

- [X] T008 [US1] Run `streamer` (with 25 sample chunks) -> `rsudp-rust` and verify "Triggered" log message for `fdsnws.mseed` data

## Dependencies

- T004 depends on failure/success of T001.
- T006 depends on T004.

## Parallel Execution Examples

- T001 (Test creation) and T003 (Code inspection) can start immediately.

## Implementation Strategy

1. **Phase 1**: Prove the bug with a unit test.
2. **Phase 2**: Fix the logic.
3. **Phase 3**: Verify with integration test.
