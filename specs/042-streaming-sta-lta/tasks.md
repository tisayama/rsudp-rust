# Tasks: Streaming STA/LTA Trigger Calculation

**Input**: Design documents from `/specs/042-streaming-sta-lta/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: Included — existing integration test must be updated and Python verification scripts must pass.

**Organization**: Tasks are grouped by user story. This is a single-file refactoring (`trigger.rs`) with integration test update.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: No setup needed — existing project, no new dependencies, no new files

*(No tasks — project structure and dependencies are already in place)*

---

## Phase 2: Foundational (StaLtaState Data Model Change)

**Purpose**: Modify the `StaLtaState` struct to support streaming state. This MUST be complete before any user story implementation.

**⚠️ CRITICAL**: The struct change is the foundation for all subsequent streaming logic.

- [x] T001 Modify `StaLtaState` struct in `rsudp-rust/src/trigger.rs`: remove `buffer: VecDeque<f64>` field, add `filters: Vec<Biquad>` (initialized from `butter_bandpass_sos()`), `sta: f64` (init 0.0), `lta: f64` (init 1e-99), `sample_count: usize` (init 0). Remove `use std::collections::VecDeque` import. Update the `or_insert_with` closure in `add_sample()` to initialize these new fields (call `butter_bandpass_sos(4, self.config.highpass, self.config.lowpass, 100.0)` for filters). Remove `total_needed` and buffer capacity calculation (lines 112-113). Keep all other fields (`triggered`, `max_ratio`, `last_timestamp`, `exceed_start`, `is_exceeding`) unchanged.

- [x] T002 Update `TriggerManager::new()` log message in `rsudp-rust/src/trigger.rs`: change "Slice Mode" to "Streaming Mode" in the `info!()` call on line 103.

**Checkpoint**: Code should compile (with warnings about unused fields) after this phase. Buffer-related code in `add_sample()` will be broken — that's expected, fixed in Phase 3.

---

## Phase 3: User Story 1 — Accurate Earthquake Alert Reset Timing (Priority: P1) 🎯 MVP

**Goal**: Replace slice-based STA/LTA calculation with streaming (incremental) calculation that matches ObsPy `recursive_sta_lta` behavior. ALARM-to-RESET for `shindo0.mseed` should be 72s ±5s.

**Independent Test**: Stream `references/mseed/shindo0.mseed` through the system, measure ALARM-to-RESET duration = 72s ±5s, peak ratio ≈ 4.5.

### Implementation for User Story 1

- [x] T003 [US1] Refactor `add_sample()` in `rsudp-rust/src/trigger.rs` to replace slice calculation (lines 128-161) with streaming calculation. Remove: `state.buffer.push_back(sample)`, buffer size limiting (`while state.buffer.len() > total_needed`), buffer length check (`if state.buffer.len() < nlta`), filter creation (`let mut filters = butter_bandpass_sos(...)`), energy vector allocation and loop (`let mut energies = ...`, `for &s in &state.buffer`), and STA/LTA loop over energies (`for i in 1..ndat`). Replace with single-sample streaming logic: (1) apply bandpass filter through persistent `state.filters`: `let mut val = sample; for section in &mut state.filters { val = section.process(val); }`, (2) compute energy: `let energy = val * val`, (3) update STA: `let csta = 1.0 / (self.config.sta_sec * 100.0); state.sta = csta * energy + (1.0 - csta) * state.sta`, (4) update LTA: `let clta = 1.0 / (self.config.lta_sec * 100.0); state.lta = clta * energy + (1.0 - clta) * state.lta`, (5) increment `state.sample_count += 1`.

- [x] T004 [US1] Implement warmup suppression in `add_sample()` in `rsudp-rust/src/trigger.rs`: after STA/LTA update (T003), compute ratio. If `state.sample_count < (self.config.lta_sec * 100.0) as usize` then `ratio = 0.0` (warmup period), else `ratio = state.sta / state.lta`. Remove `nlta` and `nsta` local variables (now computed inline). The warmup `return None` from the buffer check (`if state.buffer.len() < nlta`) is replaced by setting ratio to 0.0 — the trigger logic naturally won't fire with ratio=0.0 since threshold > 0.

**Checkpoint**: At this point, the core streaming STA/LTA should be functional. Trigger logic below the ratio calculation (lines 163-207) is UNCHANGED. Build with `cargo build --manifest-path rsudp-rust/Cargo.toml` to verify compilation.

---

## Phase 4: User Story 2 — Continuous Operation Without State Corruption (Priority: P2)

**Goal**: Ensure gap detection resets streaming state cleanly, and no memory grows over time. System can run for days without drift.

**Independent Test**: Verify that after a >1s gap, filter state and STA/LTA reset to initial values. Verify no VecDeque or growing allocation remains.

### Implementation for User Story 2

- [x] T005 [US2] Update gap detection in `add_sample()` in `rsudp-rust/src/trigger.rs`: replace `state.buffer.clear()` (line 123) with streaming state reset: `for bq in &mut state.filters { bq.s1 = 0.0; bq.s2 = 0.0; } state.sta = 0.0; state.lta = 1e-99; state.sample_count = 0;`. This resets Biquad filter state (s1/s2 only, preserving coefficients b0/b1/b2/a1/a2), STA/LTA values, and warmup counter. Filter coefficients remain constant.

**Checkpoint**: Gap detection should now reset streaming state. Build and verify: `cargo build --manifest-path rsudp-rust/Cargo.toml`.

---

## Phase 5: User Story 3 — Preserved Alert Behavior and Integration (Priority: P3)

**Goal**: All existing trigger events (ALARM, RESET, STATUS) and downstream integrations continue to work identically.

**Independent Test**: Run integration test and E2E test — ALARM/RESET/STATUS events emitted correctly.

### Implementation for User Story 3

- [x] T006 [US3] Update integration test in `rsudp-rust/tests/integration_alert.rs`: adjust warmup phase to account for streaming behavior. Change warmup loop from 1000 iterations to `nlta + 500` iterations (where nlta = lta_sec * 100 = 1000, so use 1500) to ensure LTA is fully warmed up before high-amplitude injection. Increase high-amplitude injection loop from 100 to 200 iterations to ensure trigger fires (streaming EMA needs more samples to raise ratio above threshold). Increase reset loop from 2000 to 3000 iterations. Add `assert_eq!` checks instead of bare `matches!` macro calls (current test uses `matches!()` as statements, not assertions — this is a latent bug).

**Checkpoint**: `cargo test --manifest-path rsudp-rust/Cargo.toml` should pass all tests.

---

## Phase 6: Polish & Verification

**Purpose**: Final validation against Python reference and code quality checks

- [x] T007 Run `cargo clippy --manifest-path rsudp-rust/Cargo.toml` and fix any warnings in `rsudp-rust/src/trigger.rs`
- [x] T008 Run `cargo test --manifest-path rsudp-rust/Cargo.toml` to verify all tests pass
- [x] T009 Run E2E verification: build release (`cargo build --release --manifest-path rsudp-rust/Cargo.toml`), start rsudp-rust with test config, stream `references/mseed/shindo0.mseed` via streamer, verify ALARM-to-RESET ≈ 72s ±5s in output logs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No tasks — skip
- **Phase 2 (Foundational)**: No dependencies — start immediately. T001 → T002 sequential (same file, same struct)
- **Phase 3 (US1)**: Depends on Phase 2 completion. T003 → T004 sequential (T004 uses state from T003)
- **Phase 4 (US2)**: Depends on Phase 2 completion. Can run in parallel with Phase 3 (modifies different section of `add_sample()`) but recommended sequential since gap detection references the new fields from T001
- **Phase 5 (US3)**: Depends on Phase 3 + Phase 4 completion (different file, but needs streaming logic to be correct for test to pass)
- **Phase 6 (Polish)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational (Phase 2) only — core algorithm change
- **User Story 2 (P2)**: Depends on Foundational (Phase 2) — gap detection changes the same new fields
- **User Story 3 (P3)**: Depends on US1 + US2 — integration test validates the complete streaming behavior

### Critical Path

```
T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009
```

This is a strictly sequential critical path because all implementation tasks (T001-T005) modify the same file (`trigger.rs`) and T006 validates their combined behavior.

### Parallel Opportunities

Limited parallelism due to single-file scope:
- T007 and T008 (clippy + test) can run after T006, but are fast enough that parallelism isn't needed
- T009 (E2E verification) depends on T008 passing

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 2: Foundational (T001-T002) — StaLtaState struct change
2. Complete Phase 3: User Story 1 (T003-T004) — Streaming calculation
3. **STOP and VALIDATE**: `cargo build` compiles, manual test with shindo0.mseed shows ~72s RESET

### Incremental Delivery

1. T001-T002: Struct change → compiles with warnings
2. T003-T004: Streaming STA/LTA → core fix complete, ALARM-to-RESET = 72s ±5s
3. T005: Gap detection → continuous operation safe
4. T006: Integration test → automated regression coverage
5. T007-T009: Polish → production-ready

### Single Developer Strategy

Execute T001 through T009 sequentially. Total scope: ~150 lines changed in `trigger.rs`, ~20 lines changed in `integration_alert.rs`. Estimated: single focused session.

---

## Notes

- All implementation is in a single file (`rsudp-rust/src/trigger.rs`) — no parallel file editing
- The `add_sample()` public API signature is unchanged — no callers need updating
- The trigger logic (threshold/reset/duration/status) below the ratio calculation is UNTOUCHED
- `Biquad` struct and `butter_bandpass_sos()` function are UNCHANGED (only usage changes)
- `AlertEvent`, `AlertEventType`, `TriggerConfig` are UNCHANGED
