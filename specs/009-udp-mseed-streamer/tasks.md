# Tasks: UDP MiniSEED Streamer for Simulation

**Feature Branch**: `009-udp-mseed-streamer`
**Implementation Strategy**: MVP first (Phase 1-3) followed by performance and UX improvements.

## Phase 1: Setup

- [X] T001 Add `streamer` binary target to `rsudp-rust/Cargo.toml`
- [X] T002 Create skeleton CLI structure with `clap` in `rsudp-rust/src/bin/streamer.rs`

## Phase 2: Foundational (Indexing & Timing)

- [X] T003 Implement MiniSEED file indexing (header scanning) in `rsudp-rust/src/bin/streamer.rs`
- [X] T004 [P] Implement high-precision drift-correction clock logic in `rsudp-rust/src/bin/streamer.rs`
- [X] T005 [P] Expose required record-level parsing functions in `rsudp-rust/src/parser/mseed.rs`

## Phase 3: [US1] Real-time earthquake simulation (Priority: P1)

**Goal**: Stream a MiniSEED file at its original pace as decoded rsudp packets.
**Independent Test**: Use a 10s MiniSEED file. Verify the streamer takes 10s (+/- jitter) and receiver gets valid JSON packets.

- [X] T006 [US1] Implement Steim2-to-rsudp JSON serialization logic in `rsudp-rust/src/bin/streamer.rs`
- [X] T007 [US1] Implement UDP packet transmission in `rsudp-rust/src/bin/streamer.rs`
- [X] T008 [US1] Implement the primary playback loop (sorted by time) in `rsudp-rust/src/bin/streamer.rs`
- [X] T009 [US1] Create integration test for 1x speed delivery in `rsudp-rust/tests/integration_streamer.rs`

## Phase 4: [US2] High-speed data replay (Priority: P2)

**Goal**: Accelerate playback using a speed multiplier.
**Independent Test**: Stream 60s of data at 10x speed. Verify completion in ~6s.

- [X] T010 [US2] Integrate speed multiplier into the playback clock in `rsudp-rust/src/bin/streamer.rs`
- [X] T011 [US2] Add `--speed` CLI option and validation in `rsudp-rust/src/bin/streamer.rs`
- [X] T012 [US2] Add 10x speed test case to `rsudp-rust/tests/integration_streamer.rs`

## Phase 5: [US3] Looping simulation (Priority: P3)

**Goal**: Support indefinite replay by restarting from the beginning.
**Independent Test**: Run with `--loop`. Verify transmission continues after the first file cycle ends.

- [X] T013 [US3] Implement index reset and session clock restart in `rsudp-rust/src/bin/streamer.rs`
- [X] T014 [US3] Add `--loop` CLI flag in `rsudp-rust/src/bin/streamer.rs`

## Phase 6: Polish & Cross-cutting Concerns

- [X] T015 [P] Implement graceful shutdown (Ctrl+C) handling in `rsudp-rust/src/bin/streamer.rs`
- [X] T016 [P] Add progress logging and ETA display in `rsudp-rust/src/bin/streamer.rs`

## Dependencies

- Phase 2 (Foundational) must be complete before any User Stories.
- [US1] is a prerequisite for [US2] and [US3].

## Parallel Execution Examples

- **Foundational**: T004 (Clock) and T005 (Parser) can be developed in parallel after T003 (Indexing) logic is defined.
- **US1**: T006 (Serialization) and T007 (UDP) can be developed in parallel once the index structure is stable.
- **Polish**: T015 and T016 are independent of each other.
