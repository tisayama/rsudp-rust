# Tasks: SNS Posting for Seismic Alerts

**Feature**: SNS Posting for Seismic Alerts
**Status**: Completed
**Implementation Strategy**: Implement a modular, trait-based SNS client architecture. Start with the foundational trait and S3 helper, followed by Discord (P1), then LINE/Google Chat (P2), and finally Amazon SNS (P3). All notifications are handled asynchronously to protect the real-time data pipeline.

## Phase 1: Setup
Goal: Project initialization and dependency management.

- [X] T001 Add `reqwest`, `aws-sdk-s3`, `aws-sdk-sns`, and `aws-config` to `rsudp-rust/Cargo.toml`
- [X] T002 Create `rsudp-rust/src/web/sns/` directory and initialize `rsudp-rust/src/web/sns/mod.rs`

## Phase 2: Foundational
Goal: Define core traits and shared utilities.

- [X] T003 Define `NotificationEvent` and `SNSProvider` trait in `rsudp-rust/src/web/sns/mod.rs`
- [X] T004 Implement `S3Client` for `public-read` image uploads in `rsudp-rust/src/web/sns/s3.rs`

## Phase 3: [US1] Discord Integration (Priority: P1)
Goal: Immediate rich notifications on Discord.
**Independent Test**: Configure Discord Webhook and verify text on Trigger and image on Reset using `cargo test`.

- [X] T005 [P] [US1] Implement `DiscordProvider` with text and multipart image support in `rsudp-rust/src/web/sns/discord.rs`
- [X] T006 [US1] Integrate `SNSManager` into `rsudp-rust/src/pipeline.rs` to handle Discord Trigger/Reset events

## Phase 4: [US2] LINE & Google Chat Integration (Priority: P2)
Goal: Broad messaging coverage with S3-hosted images.
**Independent Test**: Configure LINE Messaging API and S3, then verify image-linked notifications using `cargo test`.

- [X] T007 [P] [US2] Implement `LineProvider` with Messaging API support in `rsudp-rust/src/web/sns/line.rs`
- [X] T008 [P] [US2] Implement `GChatProvider` with webhook support in `rsudp-rust/src/web/sns/gchat.rs`
- [X] T009 [US2] Enable S3 upload workflow and provider orchestration in `rsudp-rust/src/web/sns/mod.rs`

## Phase 5: [US3] Amazon SNS Integration (Priority: P3)
Goal: Low-bandwidth text notifications.
**Independent Test**: Configure AWS SNS Topic and verify SMS/Notification reception.

- [X] T010 [P] [US3] Implement `AwsSnsProvider` using AWS SDK in `rsudp-rust/src/web/sns/aws_sns.rs`
- [X] T011 [US3] Finalize `SNSManager` orchestration for all providers in `rsudp-rust/src/web/sns/mod.rs`

## Phase 6: Polish & Verification
Goal: Ensure robustness and performance.

- [X] T012 Ensure non-blocking execution using `tokio::spawn` in `rsudp-rust/src/pipeline.rs`
- [X] T013 Verify error logging for rate limits and credential failures in all providers

## Dependencies

1. US1 depends on Phase 1 and 2.
2. US2 depends on Phase 1, 2, and the S3 helper (T004).
3. US3 depends on Phase 1 and 2.
4. Phase 6 depends on all User Stories being completed.

## Parallel Execution Examples

- T005 (Discord), T007 (LINE), T008 (GChat), and T010 (Amazon SNS) can be developed in parallel as they implement the same trait in different files.

## Implementation Strategy

1. **Phase 1 & 2**: Establish the infrastructure.
2. **Phase 3 (MVP)**: Deliver Discord notifications as the first functional slice.
3. **Phase 4 & 5**: Iteratively add other platforms.
4. **Phase 6**: Hardening and non-functional verification.