# Tasks: Automated E2E Alert Triggering Test

**Feature**: Automated E2E Alert Triggering Test
**Status**: Pending
**Implementation Strategy**: Add necessary test dependencies first, then implement the test harness incrementally.

## Phase 1: Setup
Goal: Configure dependencies.

- [ ] T001 Add `regex` (and `port_selector` if needed) to `rsudp-rust/Cargo.toml` as dev-dependencies

## Phase 2: Foundational
Goal: Define test utilities for process management and port selection.

- [ ] T002 [US1] Create `rsudp-rust/tests/e2e_alert.rs` with helper function to find a free UDP port
- [ ] T003 [US1] Implement `BackgroundProcess` struct in `e2e_alert.rs` that implements `Drop` to kill the child process automatically

## Phase 3: [US1] E2E Integration Test for Alert Triggering (Priority: P1)
Goal: Implement the core test logic.

- [ ] T004 [US1] Implement the main test function `test_e2e_alert_triggering` in `rsudp-rust/tests/e2e_alert.rs`
- [ ] T005 [US1] Implement `rsudp-rust` spawning logic with dynamic port and stdout capture in `e2e_alert.rs`
- [ ] T006 [US1] Implement `streamer` spawning logic with 100x speed in `e2e_alert.rs`
- [ ] T007 [US1] Implement assertion logic: read stdout for "ALARM" and check for PNG file generation in `e2e_alert.rs`

## Phase 4: Integration & Verification
Goal: Verify the test itself passes.

- [ ] T008 [US1] Run `cargo test --test e2e_alert` and verify it passes with current `master` code

## Dependencies

- T004 depends on T002 and T003.
- T005-T007 are parts of T004.

## Parallel Execution Examples

- T001 can be done independently.

## Implementation Strategy

1. **Phase 1**: Add deps.
2. **Phase 2**: Scaffold test file.
3. **Phase 3**: Write test logic.
4. **Phase 4**: Run it.
