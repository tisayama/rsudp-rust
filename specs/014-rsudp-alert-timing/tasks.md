# Tasks: rsudp準拠のアラート投稿タイミング実装

**Feature Branch**: `014-rsudp-alert-timing`
**Implementation Strategy**: Refactor the alert notification flow to trigger based on a timer relative to the start of the event, ensuring optimal visual positioning of the seismic waveform.

## Phase 1: Setup & Data Models

- [x] T001 [P] Add `window_seconds` and `save_pct` fields to `PlotSettings` struct in `rsudp-rust/src/web/stream.rs`
- [x] T002 [P] Add `save_pct` field to `AlertSettings` struct in `rsudp-rust/src/web/alerts.rs`
- [x] T003 [P] Update `PlotSettings` and `AlertSettings` interfaces in `webui/lib/types.ts` to match backend changes

## Phase 2: Foundational (Pipeline Refactoring)

- [x] T004 [US1] Remove snapshot generation and notification logic from `AlertEventType::Reset` branch in `rsudp-rust/src/pipeline.rs`
- [x] T005 [US1] Implement background task spawning (`tokio::spawn`) within the `AlertEventType::Trigger` branch in `rsudp-rust/src/pipeline.rs`
- [x] T006 [US1] Implement delay logic (`sleep`) using `window_seconds * save_pct` inside the background task in `rsudp-rust/src/pipeline.rs`

## Phase 3: [US1] Timer-based Snapshot & Notification (Priority: P1)

- [x] T007 [US1] Update background task to extract the waveform window and calculate max intensity at the scheduled execution time in `rsudp-rust/src/pipeline.rs`
- [x] T008 [US1] Trigger `generate_snapshot` and `send_reset_email` from the background task in `rsudp-rust/src/pipeline.rs`
- [x] T009 [US1] Broadcast `AlertEnd` and update history from the background task in `rsudp-rust/src/pipeline.rs`

## Phase 4: [US2] Configuration & UI (Priority: P2)

- [x] T010 [US2] Add `--window-seconds` and `--save-pct` CLI arguments to `rsudp-rust/src/main.rs`
- [x] T011 [US2] Implement `save_pct` configuration in `webui/components/AlertSettingsPanel.tsx`
- [x] T012 [US2] Ensure `window_seconds` in `webui/src/app/page.tsx` is synced with backend `PlotSettings`


## Phase 5: Polish & Validation

- [x] T013 [P] Verify correct trigger alignment (approximately 70% from right) in generated PNGs
- [x] T014 [P] Cleanup unused variables related to the old `RESET` trigger logic in `pipeline.rs`

## Dependencies

- Phase 1 must be completed before any other phase.
- Phase 2 and 3 are the core logic implementation.
- Phase 4 extends the feature with user configuration.

## Parallel Execution Examples

- **Models**: T001, T002, and T003 can be implemented simultaneously.
- **UI**: T011 and T012 can be developed in parallel with backend logic once T003 is done.
- **Verification**: T013 and T014 can be handled together during final polish.
