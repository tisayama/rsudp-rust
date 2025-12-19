# Tasks: Pure Rust MiniSEED Ingestion

**Input**: Design documents from `/specs/005-pure-rust-mseed/`
**Prerequisites**: plan.md, spec.md, data-model.md, research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Transition dependencies and establish pure-rust module structure.

- [X] T001 Remove `mseed` from `rsudp-rust/Cargo.toml` and add `byteorder` and `thiserror`.
- [X] T002 [P] Create `rsudp-rust/src/parser/header.rs` and `rsudp-rust/src/parser/steim.rs` files.
- [X] T003 [P] Update `rsudp-rust/src/parser/mod.rs` to expose `header` and `steim` modules.

---

## Phase 2: Foundational (Decompression & Header Parsing)

**Purpose**: Implement the core bit-level logic for MiniSEED 2.

- [X] T004 [P] Implement 48-byte Fixed Section Data Header (FSDH) parsing in `rsudp-rust/src/parser/header.rs`.
- [X] T005 [P] Implement `BTIME` (10-byte) to `chrono::DateTime<Utc>` conversion in `rsudp-rust/src/parser/header.rs`.
- [X] T006 Implement Steim Control Word decoding (2-bit flags) in `rsudp-rust/src/parser/steim.rs`.
- [X] T007 [P] Implement Steim1 difference decoding in `rsudp-rust/src/parser/steim.rs`.
- [X] T008 Implement Steim2 variable-bit difference decoding (all modes: 1x30, 2x15, 3x10, 5x6, 6x5, 7x4) in `rsudp-rust/src/parser/steim.rs`.

---

## Phase 3: User Story 1 - Build without C Toolchain (Priority: P1) 🎯 MVP

**Goal**: Achieve a pure-rust build by removing all FFI dependencies.

**Independent Test**: `cargo build` succeeds on a system where `clang` has been removed or disabled.

### Implementation for User Story 1

- [X] T009 [US1] Remove all imports and usages of the `mseed` crate from `rsudp-rust/src/parser/mseed.rs` (or replace the file entirely).
- [X] T010 [US1] Implement a stub `PureRustParser` in `rsudp-rust/src/parser/mod.rs` that satisfies the existing `TraceSegment` interface.
- [X] T011 [US1] Verify that `cargo build` and `cargo check` pass without any C compiler requirement.

---

## Phase 4: User Story 2 - Maintain Data Accuracy (Priority: P1)

**Goal**: Fully integrate the pure-rust parser and verify numerical correctness.

**Independent Test**: Running the simulation with `references/mseed/fdsnws.mseed` produces identical results to the previous implementation.

### Implementation for User Story 2

- [ ] T012 [US2] Integrate `header.rs` and `steim.rs` into the main parsing loop in `rsudp-rust/src/parser/mod.rs`.
- [ ] T013 [US2] Implement data reconstruction (integration of differences) and validation against Xn in `rsudp-rust/src/parser/steim.rs`.
- [ ] T014 [US2] Update `rsudp-rust/src/pipeline.rs` to use the new `PureRustParser`.
- [ ] T015 [US2] Create a dedicated regression test in `rsudp-rust/src/parser/mod.rs` that validates sample values against `references/mseed/fdsnws.mseed`.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Finalize error handling and ensure code quality.

- [ ] T016 Implement comprehensive error types using `thiserror` in `rsudp-rust/src/parser/mod.rs` for malformed records.
- [ ] T017 Add rustdoc documentation to all public parsing structures and methods.
- [ ] T018 Run `cargo clippy` and `cargo fmt` on the `rsudp-rust/` directory and resolve all issues.

---

## Dependencies & Execution Order

- **Phase 1**: No dependencies.
- **Phase 2**: Depends on Phase 1.
- **Phase 3 (MVP)**: Depends on Phase 1. Can run in parallel with Phase 2 if using stubs.
- **Phase 4**: Depends on Phase 2 and Phase 3 completion.
- **Phase 5**: Depends on Phase 4.

## Implementation Strategy

### MVP First (User Story 1)

1. Remove C dependencies first to ensure build portability.
2. Use stubs to maintain the pipeline interface while building.

### Incremental Delivery

1. Setup structure (Phase 1).
2. Bit-level logic (Phase 2).
3. Build verification (Phase 3).
4. Full integration and accuracy test (Phase 4).
5. Final cleanup (Phase 5).
