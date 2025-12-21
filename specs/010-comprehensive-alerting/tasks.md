# Tasks: Comprehensive Alerting System

**Feature Branch**: `010-comprehensive-alerting`
**Implementation Strategy**: Multi-phase rollout starting with real-time UI notifications, followed by automated snapshots and email integration.

## Phase 1: Setup

- [x] T001 Add `plotters`, `lettre`, and `tower-http` dependencies to `rsudp-rust/Cargo.toml`
- [x] T002 Create local storage directory for alert snapshots in `rsudp-rust/alerts/`
- [x] T003 Add placeholder alert sound file to `webui/public/sounds/alert.wav`

## Phase 2: Foundational (Infrastructure)

- [x] T004 Define `AlertEvent` and `AlertSettings` models in `rsudp-rust/src/web/alerts.rs`
- [x] T005 Implement in-memory alert history manager with 24h retention in `rsudp-rust/src/web/history.rs`
- [x] T006 [P] Add REST endpoints for alert history and settings in `rsudp-rust/src/web/routes.rs`
- [x] T007 [P] Implement static file serving for the alerts directory in `rsudp-rust/src/web/routes.rs`

## Phase 3: [US1] Real-time visual and audio notifications (Priority: P1)

**Goal**: Immediate notification of seismic events in the browser.
**Independent Test**: Simulate a trigger in the pipeline and verify the WebUI background flashes and audio plays.

- [x] T008 [US1] Extend WebSocket protocol with `AlertStart` and `AlertEnd` message types in `rsudp-rust/src/web/stream.rs`
- [x] T009 [US1] Update pipeline trigger logic to broadcast alert messages via `rsudp-rust/src/pipeline.rs`
- [x] T010 [US1] Implement `useAlerts` hook for state management and audio playback in `webui/hooks/useAlerts.ts`
- [x] T011 [US1] Integrate alert visual feedback (background flashing) in `webui/src/app/page.tsx`

## Phase 4: [US2] Event snapshot automatic generation (Priority: P2)

**Goal**: Automatic PNG creation of waveform during alerts.
**Independent Test**: Trigger an alert and verify a PNG file appears in the `alerts/` folder.

- [x] T012 [US2] Implement a sliding buffer for the last 60s of waveform data in `rsudp-rust/src/pipeline.rs`
- [x] T013 [US2] Implement PNG plotting logic using `plotters` in `rsudp-rust/src/web/alerts.rs`
- [x] T014 [US2] Trigger snapshot generation on alert `Reset` event in `rsudp-rust/src/pipeline.rs`
- [x] T015 [US2] Create alert history page to display events and images in `webui/src/app/history/page.tsx`

## Phase 5: [US3] Layered email notifications (Priority: P3)

**Goal**: Tiered email alerts for immediate awareness and detailed reporting.
**Independent Test**: Trigger an alert and verify receipt of both "Trigger" and "Summary" emails.

- [x] T016 [US3] Implement SMTP transport backend using `lettre` in `rsudp-rust/src/web/alerts.rs`
- [x] T017 [US3] Implement immediate "Trigger" email notification in `rsudp-rust/src/web/alerts.rs`
- [x] T018 [US3] Implement detailed "Reset" email with max ratio and snapshot link in `rsudp-rust/src/web/alerts.rs`

## Phase 6: Polish & Cross-cutting Concerns

- [x] T019 [P] Offload image generation and email sending to `tokio::spawn` tasks in `rsudp-rust/src/web/alerts.rs`
- [x] T020 [P] Implement settings UI for toggling audio/email and configuring SMTP in `webui/components/SettingsPanel.tsx`
- [x] T021 [P] Add automatic deletion logic for PNG files older than 24h in `rsudp-rust/src/web/history.rs`

## Dependencies

- Phase 2 (Foundational) must be completed before User Story phases.
- [US1] must be completed before [US2] to provide the trigger infrastructure.
- [US2] provides the image URLs required for the detailed emails in [US3].

## Parallel Execution Examples

- **Foundational**: T006 (Endpoints) and T007 (Static Serving) can be done in parallel once models (T004) are defined.
- **US1**: T010 (Hook) and T011 (UI) can be done in parallel once the WebSocket protocol (T008) is updated.
- **Polish**: T020 (Settings UI) can be developed independently of background task optimization (T019).

## Implementation Strategy

1. **Phase 1-3 (MVP)**: Focus on getting the signal from the Rust backend to the browser with sound and visual cues. This provides the highest immediate value.
2. **Phase 4**: Add the archival capability (PNGs). This is critical for scientific verification.
3. **Phase 5**: Extend notifications outside the browser (Email).
4. **Phase 6**: Ensure system robustness and long-term stability (Cleanup, Async offloading).
