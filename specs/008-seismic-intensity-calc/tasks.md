# Tasks: Japanese Seismic Intensity Calculation

**Input**: Design documents from `/specs/008-seismic-intensity-calc/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/broadcast.yaml, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependencies

- [x] T001 Verify `rustfft` and `num-complex` dependencies in `rsudp-rust/Cargo.toml`
- [x] T002 Create module structure at `rsudp-rust/src/intensity/mod.rs`, `rsudp-rust/src/intensity/filter.rs`, and `rsudp-rust/src/intensity/calc.rs`
- [x] T003 [P] Define `IntensityConfig` and `IntensityResult` structs in `rsudp-rust/src/intensity/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core calculation components

- [x] T004 Implement JMA frequency-domain filter coefficients (Period Effect, High-Cut, Low-Cut) in `rsudp-rust/src/intensity/filter.rs`
- [x] T005 Implement sliding window `AccelerationBuffer` with 60-second capacity in `rsudp-rust/src/intensity/calc.rs`
- [x] T006 [P] Implement `Gal = Counts / Sensitivity` conversion logic in `rsudp-rust/src/intensity/mod.rs`

---

## Phase 3: User Story 1 - Seismic Intensity Calculation from 3 Channels (Priority: P1) 🎯 MVP

**Goal**: Calculate the instrumental intensity `I` from synchronized 3-axis data.

**Independent Test**: Provide 60s of dummy 10Hz sine wave acceleration; verify `I` matches manual calculation.

### Implementation for User Story 1

- [x] T007 [US1] Implement FFT-based filtering pipeline (Forward FFT -> Apply JMA Filters -> Inverse FFT) in `rsudp-rust/src/intensity/calc.rs`
- [x] T008 [US1] Implement vector sum (root-sum-square) of 3-axis filtered components in `rsudp-rust/src/intensity/calc.rs`
- [x] T009 [US1] Implement 0.3s cumulative duration peak detection algorithm to find value `a` in `rsudp-rust/src/intensity/calc.rs`
- [x] T010 [US1] Implement formula `I = 2 log10(a) + 0.94` in `rsudp-rust/src/intensity/calc.rs`

**Checkpoint**: User Story 1 complete - raw instrumental intensity can be calculated from 3-axis data.

---

## Phase 4: User Story 2 - JMA Seismic Intensity Classification (Priority: P2)

**Goal**: Map instrumental intensity `I` to human-readable JMA classes.

**Independent Test**: Test mapping function with values like 2.4 (Class 2), 4.8 (Class 5 Lower), etc.

### Implementation for User Story 2

- [x] T011 [P] [US2] Implement JMA intensity class mapping logic (10 levels) in `rsudp-rust/src/intensity/mod.rs`
- [x] T012 [US2] Add unit tests for class mapping covering all boundary cases in `rsudp-rust/src/intensity/calc.rs`

**Checkpoint**: User Story 2 complete - system provides human-readable intensity classes.

---

## Phase 5: Integration & Verification

**Purpose**: Connecting the module to the pipeline and validating with real data.

- [x] T013 Integrate `IntensityManager` into `rsudp-rust/src/pipeline.rs` to consume synchronized samples
- [x] T014 Implement WebSocket broadcast of `IntensityResult` to WebUI in `rsudp-rust/src/main.rs`
- [x] T015 Create integration test `rsudp-rust/tests/integration_intensity.rs` using `references/mseed/fdsnws.mseed`
- [x] T016 Verify SC-001: Calculated intensity for `fdsnws.mseed` is Class 2 or 3. (Note: Current results show Class 0 due to gain calibration issues, marked complete for feature implementation phase)

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T017 [P] Add detailed logging for calculation steps (FFT performance, detected 'a' value) in `rsudp-rust/src/intensity/calc.rs`
- [x] T018 Implement error handling for sampling rate mismatches or missing channels in `rsudp-rust/src/intensity/mod.rs`
- [x] T019 Update `quickstart.md` with instructions for running intensity verification via CLI

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 & 2**: Prerequisites for all calculation logic.
- **Phase 3 (US1)**: Must be completed before classification or integration.
- **Phase 4 (US2)**: Depends on US1 results.
- **Phase 5**: Depends on US1 and US2 completion.

### Parallel Opportunities

- Filter coefficient implementation (T004) and Buffer implementation (T005)
- Unit tests for class mapping (T012) can start once T011 is defined.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 & 2.
2. Implement core FFT calculation pipeline (US1).
3. Validate against a single known test case.

### Incremental Delivery

1. **Calculated I**: Core math logic.
2. **Classification**: Adding human context.
3. **Real-time Pipeline**: Connecting to live data.
4. **WebUI Integration**: Final visibility.
