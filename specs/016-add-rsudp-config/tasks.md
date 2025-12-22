# Tasks: Implement rsudp-compatible Configuration System

**Feature**: Implement rsudp-compatible Configuration System
**Status**: Completed
**Implementation Strategy**: MVP (User Story 1 - TOML) first, followed by YAML and CLI/Env priority merging.

## Phase 1: Setup
Goal: Initialize dependencies and basic structure.

- [X] T001 Add `toml`, `serde_yaml`, and `config` dependencies to `rsudp-rust/Cargo.toml`
- [X] T002 Create `rsudp-rust/src/settings.rs` and export it in `rsudp-rust/src/lib.rs`

## Phase 2: Foundational
Goal: Define the configuration schema and default values.

- [X] T003 Define `Settings` and all sub-section structs (`PlotSettings`, `AlertSettings`, etc.) in `rsudp-rust/src/settings.rs`
- [X] T004 Implement `Default` trait for all settings structs with `rsudp` compatible values in `rsudp-rust/src/settings.rs`
- [X] T005 [P] Add Serde attributes for case-insensitive or custom field names if necessary in `rsudp-rust/src/settings.rs`

## Phase 3: [US1] Load Configuration from TOML File (Priority: P1)
Goal: Support loading and merging TOML configuration.

- [X] T006 [US1] Implement TOML parsing logic using `config` crate in `rsudp-rust/src/settings.rs`
- [X] T007 [US1] Implement standard search path logic (`~/.rsudp/settings.toml`) in `rsudp-rust/src/settings.rs`
- [X] T008 [US1] Create unit test for TOML loading and merging in `rsudp-rust/src/settings.rs`
- [X] T009 [US1] Implement warning logger for unknown fields during parsing in `rsudp-rust/src/settings.rs`

## Phase 4: [US2] Load Configuration from YAML File (Priority: P2)
Goal: Support YAML configuration format.

- [X] T010 [US2] Add YAML support to the `config` loader in `rsudp-rust/src/settings.rs`
- [X] T011 [US2] Implement search logic for `~/.rsudp/settings.yaml` with TOML priority in `rsudp-rust/src/settings.rs`
- [X] T012 [US2] Create unit test for YAML loading in `rsudp-rust/src/settings.rs`

## Phase 5: [US3] Generate Default Configuration (Priority: P3)
Goal: Allow users to dump default configuration to a file.

- [X] T013 [US3] Implement serialization logic to dump current `Settings` to TOML/YAML string in `rsudp-rust/src/settings.rs`
- [X] T014 [US3] Add `--dump-config <PATH>` flag to CLI in `rsudp-rust/src/main.rs`
- [X] T015 [US3] Create integration test for config dumping and re-loading in `rsudp-rust/tests/test_settings.rs`

## Phase 6: CLI & Environment Integration
Goal: Implement the full priority chain.

- [X] T016 Implement environment variable loading with `RUSTRSUDP_` prefix in `rsudp-rust/src/settings.rs`
- [X] T017 Update `rsudp-rust/src/main.rs` to allow `--config` path and merge CLI flags into the `Settings` object
- [X] T018 [P] Implement validation logic for critical fields (port range, etc.) in `rsudp-rust/src/settings.rs`

## Phase 7: Polish & Refactoring
Goal: Ensure consistency and clean integration.

- [X] T019 Update `rsudp-rust/src/main.rs` to use the new `Settings` object throughout the application initialization
- [X] T020 [P] Add documentation comments to all public configuration fields in `rsudp-rust/src/settings.rs`

## Dependencies

- Story completion order: [US1] -> [US2] -> [US3]
- Phase 6 (CLI/Env) depends on Phase 3 ([US1]) being functional.

## Parallel Execution Examples

- **User Story 1**: T007 and T009 can be done in parallel once T006 is drafted.
- **Foundation**: T005 can be done alongside T003/T004 if field names are known.
- **Refinement**: T018 and T020 can be done anytime after Phase 2.

## Implementation Strategy

1. **Phase 1-2**: Critical for any progress.
2. **Phase 3 (MVP)**: Focus on TOML loading from a file first. This provides the most value immediately.
3. **Phase 6 (Early)**: Integrate basic CLI/Env merging early to ensure the architecture holds for the full priority chain.