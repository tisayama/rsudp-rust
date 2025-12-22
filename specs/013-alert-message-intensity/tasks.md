# Tasks: Intensity Inclusion in Alert Messages

**Feature Branch**: `013-alert-message-intensity`
**Implementation Strategy**: Enhance the backend to generate descriptive Japanese intensity messages upon alert completion and ensure these are propagated to emails and the WebUI.

## Phase 1: Setup & Data Models

- [x] T001 Add `message` field to `AlertEvent` struct in `rsudp-rust/src/web/alerts.rs`
- [x] T002 Update `WsMessage::AlertEnd` variant to include `message` string in `rsudp-rust/src/web/stream.rs`
- [x] T003 [P] Update `AlertEvent` interface in `webui/lib/types.ts` to include `message` field
- [x] T004 [P] Update `WsMessage` type in `webui/lib/types.ts` to include `message` in `AlertEnd` payload

## Phase 2: Foundational (Formatting Logic)

- [x] T005 Implement `format_shindo_message` helper function in `rsudp-rust/src/web/alerts.rs` (handles Shindo 0 and 1-7 with weak/strong suffixes)

## Phase 3: [US1] Notification Integration (Priority: P1)

**Goal**: Include the intensity message in emails and immediate logs.
**Independent Test**: Trigger an alert and verify the Reset email and console log contain the correct intensity message.

- [x] T006 [US1] Update `run_pipeline` in `rsudp-rust/src/pipeline.rs` to generate the message using the final max intensity on `Reset`
- [x] T007 [US1] Pass the generated message to `web_state.broadcast_alert_end` in `rsudp-rust/src/pipeline.rs`
- [x] T008 [US1] Update `send_reset_email` in `rsudp-rust/src/web/alerts.rs` to accept and include the message in the email body
- [x] T009 [US1] Store the message in the in-memory history within `rsudp-rust/src/pipeline.rs`

## Phase 4: [US2] WebUI History Display (Priority: P2)

**Goal**: Display the intensity description in the WebUI history cards.
**Independent Test**: Open the Alert History page and verify that each card shows the "震度X相当..." text.

- [x] T010 [US2] Update alert card component/rendering in `webui/src/app/history/page.tsx` to display the `message` field

## Phase 5: Polish & Validation

- [x] T011 [P] Verify that "震度 5弱相当" and "震度 6強相当" are formatted correctly across all outputs
- [x] T012 [P] Ensure no regression in `Trigger` (start) notifications which should remain as-is

## Dependencies

- Phase 1 and 2 are prerequisites for Phase 3.
- Phase 3 provides the data required for Phase 4.

## Parallel Execution Examples

- **Phase 1**: T003 and T004 (frontend types) can be updated in parallel with T001 and T002 (backend types).
- **Phase 5**: Final verification tasks can be distributed.
