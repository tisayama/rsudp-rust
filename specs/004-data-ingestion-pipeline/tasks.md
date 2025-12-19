# Tasks: Data Ingestion Pipeline

**Input**: Design documents from `/specs/004-data-ingestion-pipeline/`
**Prerequisites**: plan.md, spec.md, data-model.md, research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependencies and establish module structure.

- [X] T001 Add `mseed` and `chrono` (with serde) to `rsudp-rust/Cargo.toml`.
- [X] T002 [P] Create `rsudp-rust/src/parser/mod.rs` and `rsudp-rust/src/parser/mseed.rs`.
- [X] T003 [P] Create `rsudp-rust/src/pipeline.rs` and expose it in `rsudp-rust/src/lib.rs`.

---

## Phase 2: Foundational (Entities & State Management)

**Purpose**: Define data structures and the filter state manager.

- [X] T004 [P] Define `TraceSegment` struct in `rsudp-rust/src/parser/mod.rs` based on `data-model.md`.
- [X] T005 [P] Implement `FilterManager` in `rsudp-rust/src/pipeline.rs` using a `HashMap` to manage `RecursiveStaLta` instances keyed by NSLC strings.
- [X] T006 Implement a method in `FilterManager` to retrieve or create a filter for a given NSLC.

---

## Phase 3: User Story 1 - Real-time Data Pipeline (Priority: P1) 🎯 MVP

**Goal**: Process bytes from UDP, parse MiniSEED, and update STA/LTA.

**Independent Test**: Send a MiniSEED record via `nc -u` and verify the pipeline logs the parsed segment and calculated ratio.

### Implementation for User Story 1

- [X] T007 [US1] Implement MiniSEED record parsing in `rsudp-rust/src/parser/mseed.rs` using the `mseed` crate to extract samples and metadata.
- [X] T008 [US1] Implement `run_pipeline` task in `rsudp-rust/src/pipeline.rs` that listens on an MPSC channel for raw bytes and dispatches to the parser.
- [X] T009 [US1] Integrate the parser output with the `FilterManager` to update STA/LTA filters.
- [X] T010 [US1] Update `rsudp-rust/src/main.rs` to connect the `Receiver` output channel to the pipeline input channel.

---

## Phase 4: User Story 2 - MiniSEED File Simulation (Priority: P2)

**Goal**: Ingest data from files instead of UDP.

**Independent Test**: Running `cargo run -- --file test.mseed` processes the file and outputs results.

### Implementation for User Story 2

- [X] T011 [US2] Update `rsudp-rust/src/settings.rs` to add a `--file` (`-f`) argument that accepts multiple paths.
- [X] T012 [US2] Implement a file reader in `rsudp-rust/src/main.rs` (or a helper) that reads MiniSEED files and pushes their content into the pipeline channel.
- [X] T013 [US2] Ensure the application waits for the pipeline to finish processing all file data before exiting in simulation mode.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Error handling, gap detection, and cleanup.

- [X] T014 [US1] Implement gap detection logic in `FilterManager` (`pipeline.rs`): reset filter if time gap > 10s.
- [ ] T015 [P] Add unit tests for MiniSEED parsing with various record types in `rsudp-rust/src/parser/mseed.rs`.
- [ ] T016 Run `cargo clippy` and fix warnings across the new modules.

---

## Dependencies & Execution Order

- **Phase 1**: No dependencies.
- **Phase 2**: Depends on Phase 1.
- **Phase 3 (MVP)**: Depends on Phase 2.
- **Phase 4**: Depends on Phase 3.
- **Phase 5**: Depends on Phase 3/4.

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 & 2.
2. Implement Phase 3 (Real-time Pipeline).
3. **STOP and VALIDATE**: Verify UDP to STA/LTA flow works with a simple test script.

### Parallel Team Strategy

- Developer A: Working on `parser/` (T007, T015).
- Developer B: Working on `pipeline.rs` and state management (T005, T006, T008, T009, T014).
- Developer C: Working on CLI and integration (T010, T011, T012, T013).
