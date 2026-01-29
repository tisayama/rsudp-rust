# Tasks: Fix Max Intensity Spike in Noise Data

**Feature**: `028-fix-intensity-spike`  
**Status**: Pending

## Implementation Strategy

This feature focuses on fixing a specific false positive bug in the seismic intensity calculation. The implementation will follow a rigorous reproduction-fix-verification cycle using real-world data (`normal.mseed`).

1.  **Phase 1 (Setup)**: Prepare the environment and verify the `normal.mseed` dataset.
2.  **Phase 2 (Foundational)**: Analyze the `IntensityManager` and dataset to hypothesize the root cause.
3.  **Phase 3 (User Story 1)**: Create a reproduction tool, scan for the spike around 3000s (50 min), and implement the fix.
4.  **Phase 4 (User Story 2)**: Verify the fix against the logic spec and ensure no regressions for valid data.
5.  **Phase 5 (Polish)**: Final integrated testing and documentation.

## Phase 1: Setup

- [ ] T001 Create feature branch `028-fix-intensity-spike` (already done by specify)
- [ ] T002 Verify presence of `references/mseed/normal.mseed` (60-minute dataset)

## Phase 2: Foundational (Analysis)

- [ ] T003 Analyze `rsudp-rust/src/intensity.rs` to map current JMA calculation steps (windowing, filtering, buffer mgmt)
- [ ] T004 Analyze `rsudp-rust/src/trigger.rs` to see if filter initialization logic is shared/relevant
- [ ] T005 [P] Inspect `references/mseed/normal.mseed` around T00:52:00 (approx +3000s from start T00:02:00) for data gaps, overlaps, or DC offsets using `obspy` (create temp python script if needed)

## Phase 3: Reproduce and Eliminate Intensity Spike (User Story 1)

**Goal**: Reproduce the 3.30 intensity spike at approx 3000s and eliminate it.
**Independent Test**: `repro_spike` binary shows spike before fix and no spike after fix.

- [ ] T006 [US1] Create `rsudp-rust/src/bin/repro_spike.rs` to read mseed and feed `IntensityManager`
- [ ] T007 [US1] Run `repro_spike.rs` with `normal.mseed` and log output to `logs/repro_before.log`
- [ ] T008 [US1] Analyze `logs/repro_before.log` to pinpoint exact timestamp of spike (Scanning around 3000s mark)
- [ ] T009 [US1] Implement fix in `rsudp-rust/src/intensity.rs` (Candidate: Detrend/Taper before FFT, or handle NaN/Inf)
- [ ] T010 [US1] Run `repro_spike.rs` again with fixed code and log to `logs/repro_after.log`
- [ ] T011 [US1] Verify `logs/repro_after.log` shows Max Intensity < 1.5 at the target window

## Phase 4: Verify Calculation Logic Integrity (User Story 2)

**Goal**: Ensure fix is mathematically correct and regression-free.
**Independent Test**: `cargo test` passes and `fdsnws.mseed` (valid event) still triggers correctly.

- [ ] T012 [P] [US2] Review `rsudp-rust/src/intensity.rs` against JMA standard spec (008) to ensure fix is compliant
- [ ] T013 [US2] Run regression test using `fdsnws.mseed` (known earthquake) with `repro_spike.rs` to ensure intensity is still high during event
- [ ] T014 [US2] Add unit tests for edge cases (zeros, DC offset step) in `rsudp-rust/src/intensity.rs`

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T015 Run full integrated pipeline test with `streamer` and `rsudp-rust` using `normal.mseed`
- [ ] T016 Update `GEMINI.md` with details of the fix and root cause
- [ ] T017 Clean up temporary logs (`logs/repro_*.log`) and binary (`src/bin/repro_spike.rs` if not keeping)

## Dependencies

- **Phase 3** depends on **Phase 2** analysis.
- **Phase 4** depends on **Phase 3** fix implementation.
- **Phase 5** depends on **Phase 4** verification.

## Parallel Execution Opportunities

- T005 (Data Analysis) can run parallel to T003/T004 (Code Analysis).
- T012 (Spec Review) can start once T009 (Fix) is drafted.
- T014 (Unit Tests) can be written parallel to T009 (Fix).
