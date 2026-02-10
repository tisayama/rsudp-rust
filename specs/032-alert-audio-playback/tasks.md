# Tasks: Alert Audio Playback

**Feature**: Alert Audio Playback
**Branch**: `032-alert-audio-playback`
**Status**: DRAFT

## Phase 1: Setup

*Goal: Initialize project structure and dependencies.*

**Independent Test Criteria**: Project compiles with `rodio` dependency.

- [x] T001 Add `rodio` dependency (with symphonia features) to `rsudp-rust/Cargo.toml`
- [x] T002 Create `rsudp-rust/src/sound.rs` module file in `rsudp-rust/src/sound.rs`
- [x] T003 Expose `sound` module in `rsudp-rust/src/lib.rs`
- [x] T004 Define `AlertSoundSettings` struct in `rsudp-rust/src/settings.rs` matching data model

## Phase 2: Foundational Tasks

*Goal: Implement core audio playback engine.*

**Independent Test Criteria**: Can verify audio playback logic via unit tests (mocked or headless) or simple run.

- [x] T005 Implement `AudioManager` struct with `OutputStream` persistence in `rsudp-rust/src/sound.rs`
- [x] T006 Implement `play_file` method with preemption logic (replacing sink) in `rsudp-rust/src/sound.rs`
- [x] T007 Implement global/shared `AudioController` to hold `AudioManager` state in `rsudp-rust/src/sound.rs`
- [x] T008 [P] Add error handling for missing files (log only, no crash) in `rsudp-rust/src/sound.rs`

## Phase 3: Alert Logic Integration (User Story 1)

*Goal: Connect Trigger and Reset events to audio playback.*

**Independent Test Criteria**: Trigger plays sound; Reset plays sound based on intensity.

- [x] T009 [US1] Initialize `AudioController` in `rsudp-rust/src/main.rs` and pass to pipeline
- [x] T010 [US1] Implement Trigger phase playback (immediate preemption) in `rsudp-rust/src/pipeline.rs`
- [x] T011 [US1] Implement Reset phase playback with intensity mapping logic in `rsudp-rust/src/pipeline.rs`
- [x] T012 [P] [US1] Add default reset sound fallback logic in `rsudp-rust/src/pipeline.rs`

## Phase 4: Polish & Cross-Cutting Concerns

*Goal: Finalize configuration and documentation.*

**Independent Test Criteria**: Documentation matches implementation.

- [x] T013 Update `rsudp.toml` example with `[ALERTSOUND]` section in `scripts/rsudp_settings.toml`
- [x] T014 Update `README.md` with setup instructions (ALSA, file paths) in `README.md`

## Dependencies

1. **Phase 1**: Blocks everything.
2. **Phase 2**: Blocks Phase 3.
3. **Phase 3**: Core integration.
4. **Phase 4**: Cleanup.

## Parallel Execution Examples

- **Settings & Module**: T003 and T004 can be done in parallel.
- **Trigger & Reset Logic**: T010 and T011 can be implemented in parallel once `play_file` exists.

## Implementation Strategy

1. **Core Audio**: Build `AudioManager` first to ensure we can control `rodio` sinks correctly for preemption.
2. **Integration**: Hook into the existing pipeline where Hue/SNS alerts are triggered.
3. **Config**: Ensure flexible TOML mapping works for intensity levels.
