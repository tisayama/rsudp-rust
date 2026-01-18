# Tasks: Fix Streamer UDP Packet Compatibility for rsudp

**Feature**: Fix Streamer UDP Packet Compatibility for rsudp
**Status**: Completed
**Implementation Strategy**: Direct modification of `streamer` utility to emit `rsudp`-compatible string packets instead of JSON.

## Phase 1: Setup
Goal: Verify environment and reproducibility (optional but recommended).

- [X] T001 Verify `rsudp-rust/src/bin/streamer.rs` exists and is buildable

## Phase 2: Foundational
Goal: Define the packet formatting logic.

- [X] T002 [US1] Create a helper function `format_packet(channel: &str, timestamp: f64, samples: &[i32]) -> String` in `rsudp-rust/src/bin/streamer.rs` (or a shared module if preferred) that implements the `{ 'CH', TS, S1, ... }` format
- [X] T003 [P] [US1] Write a unit test for `format_packet` to ensure it produces the exact string structure required (`{'EHZ', 123.456, 1, 2, 3}`)

## Phase 3: [US1] Streamer UDP Packet Compatibility (Priority: P1)
Goal: Replace JSON serialization with custom formatting in `streamer`.

- [X] T004 [US1] Replace `serde_json::to_string` with the new formatting logic in the main loop of `rsudp-rust/src/bin/streamer.rs`
- [X] T005 [US1] Ensure `timestamp` is passed as a float (seconds since epoch) with sufficient precision in `rsudp-rust/src/bin/streamer.rs`
- [X] T006 [US1] Ensure `channel` is wrapped in single quotes in `rsudp-rust/src/bin/streamer.rs`

## Phase 4: Integration & Verification
Goal: Verify compatibility with `rsudp`.

- [X] T007 [US1] Create an integration test script or instructions in `rsudp-rust/tests/integration_streamer.rs` that validates the packet format against a mock receiver or regression test case

## Dependencies

- All tasks are sequential within US1, but T003 (test) can be written before T002 (impl) for TDD.

## Parallel Execution Examples

- T003 (Test) and T002 (Impl) can be done in TDD cycle.

## Implementation Strategy

1. **Phase 2**: Define the formatter and test it in isolation to ensure exact string matching.
2. **Phase 3**: Swap the implementation in `streamer.rs`.
3. **Phase 4**: Verify.