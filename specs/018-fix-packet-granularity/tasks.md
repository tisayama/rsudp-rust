# Tasks: Fix UDP Packet Granularity for rsudp Compatibility

**Feature**: Fix UDP Packet Granularity for rsudp Compatibility
**Status**: Completed
**Implementation Strategy**: Direct modification of `streamer` utility to implement chunking logic with `tokio` sleep.

## Phase 1: Setup
Goal: Verify `streamer` code structure.

- [X] T001 Verify `rsudp-rust/src/bin/streamer.rs` build status and dependencies

## Phase 2: Foundational
Goal: Define CLI arguments and data structures for chunking.

- [X] T002 [US1] Add `--samples-per-packet` (default: 25) argument to `Args` struct in `rsudp-rust/src/bin/streamer.rs`
- [X] T003 [US1] Implement `chunk_samples` logic (or helper function) to split a vector of samples into smaller vectors

## Phase 3: [US1] Optimize Packet Granularity (Priority: P1)
Goal: Implement the transmission loop with precise timing.

- [X] T004 [US1] Refactor the main loop in `rsudp-rust/src/bin/streamer.rs` to iterate over chunks instead of full records
- [X] T005 [US1] Implement timestamp calculation for each chunk (start_time + (index * 1.0 / sample_rate)) in `rsudp-rust/src/bin/streamer.rs`
- [X] T006 [US1] Implement transmission delay logic: `sleep(chunk_duration / speed)` in `rsudp-rust/src/bin/streamer.rs`
- [X] T007 [P] [US1] Ensure `format_packet` handles variable chunk sizes correctly (it should, as it takes a slice)

## Phase 4: Integration & Verification
Goal: Verify smooth streaming.

- [X] T008 [US1] Update `rsudp-rust/tests/integration_streamer.rs` to verify packet frequency (if possible) or ensure no regressions in format

## Dependencies

- T004 depends on T002 and T003.
- T005 and T006 are part of the T004 refactoring.

## Parallel Execution Examples

- T007 (verification of format function) can be done in parallel with T002 (CLI args).

## Implementation Strategy

1. **Phase 2**: Add the CLI arg and define the chunking helper.
2. **Phase 3**: Rewrite the main loop to process chunks.
3. **Phase 4**: Verify.