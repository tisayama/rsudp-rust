# Tasks: Fix STA/LTA Trigger Behavior

**Feature**: Fix STA/LTA Trigger Behavior
**Status**: Completed
**Implementation Strategy**: Replace the filter with a 4th-order cascaded biquad, implement strict data dropping for warm-up, and add duration-based debouncing to match `rsudp`.

## Phase 1: Setup
Goal: Prepare for filter and logic updates.

- [X] T001 Verify existing unit tests fail or behave differently than expected (optional but good practice)

## Phase 2: Foundational
Goal: Update core logic components.

- [X] T002 Update `TriggerConfig` struct in `rsudp-rust/src/trigger.rs` to include `duration` field
- [X] T003 Update `StaLtaState` struct in `rsudp-rust/src/trigger.rs` to track warm-up status and duration timers
- [X] T004 Implement 4th-order (cascaded 2nd-order) Butterworth Bandpass Filter in `rsudp-rust/src/trigger.rs`

## Phase 3: [US1] Parity with Python Implementation (Priority: P1)
Goal: Ensure trigger behavior matches rsudp exactly.

- [X] T005 [US1] Implement `wait_pkts` equivalent warm-up logic (drop data until LTA window fills) in `rsudp-rust/src/trigger.rs`
- [X] T006 [US1] Implement duration-based debounce logic in `rsudp-rust/src/trigger.rs`
- [X] T007 [US1] Update `TriggerManager::new` and `add_sample` to use the new logic and filter

## Phase 4: Polish & Cross-Cutting Concerns
Goal: Final verification.

- [X] T008 Update `rsudp-rust/src/main.rs` to populate `duration` in `TriggerConfig` from settings
- [ ] T009 Verify fix with `streamer` using the steps in `quickstart.md`

## Dependencies

- Phase 3 depends on Phase 2 logic updates.
- T008 depends on T002.

## Parallel Execution Examples

- T004 (Filter) can be implemented independently of T002/T003 (State/Config updates).

## Implementation Strategy

1. **Filter**: Upgrade the signal processing first.
2. **State**: Add necessary fields for tracking time and warm-up.
3. **Logic**: Rewrite the decision loop to strictly follow `rsudp`.