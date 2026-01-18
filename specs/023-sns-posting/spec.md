# Feature Specification: SNS Posting for Seismic Alerts

**Feature Branch**: `023-sns-posting`  
**Created**: 2026-01-18  
**Status**: Draft  
**Input**: User description: "SNSへの投稿機能を実装してほしいです。元になる実装はrsudpを参考にしてください。Telegram, Twitter, Blueskyは実装しなくてよいです。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Discord Notification with Waveform (Priority: P1)

As a user, I want to receive a rich notification on Discord via Webhook when an earthquake is detected, including the waveform image, so that I can see the event details immediately.

**Why this priority**: Discord is the most feature-complete SNS integration in `rsudp` because it supports direct image uploads without external storage.

**Independent Test**:
1. Configure `discord.webhook_url` in settings.
2. Trigger an alert.
3. Verify a message with an embed and image appears in the Discord channel.

**Acceptance Scenarios**:
1. **Given** a Discord Webhook URL, **When** an alert is triggered, **Then** a text message MUST be posted immediately.
2. **Given** an alert concludes and a snapshot is generated, **When** image support is enabled, **Then** the PNG file MUST be uploaded directly to Discord and displayed in an embed.

---

### User Story 2 - LINE & Google Chat Notifications (Priority: P2)

As a user, I want to receive notifications on LINE and Google Chat, including waveform images via S3, to ensure broad coverage across different messaging platforms.

**Why this priority**: These platforms are widely used in professional and personal contexts in Japan.

**Independent Test**:
1. Configure LINE Messaging API tokens or Google Chat Webhook URL.
2. Configure AWS S3 credentials and bucket.
3. Trigger an alert and verify the message (with image URL) appears on the platform.

**Acceptance Scenarios**:
1. **Given** LINE/Google Chat configuration, **When** an alert occurs, **Then** a message MUST be posted.
2. **Given** S3 configuration, **When** a snapshot is generated, **Then** the image MUST be uploaded to S3 and its URL included in the LINE/Google Chat post.

---

### User Story 3 - Amazon SNS Notification (Priority: P3)

As a user, I want to receive a simple text notification via Amazon SNS (e.g., for SMS), so that I am notified even without a data connection.

**Why this priority**: Provides a low-bandwidth, reliable secondary notification channel.

**Independent Test**:
1. Configure AWS SNS Topic ARN and credentials.
2. Trigger an alert and verify an SMS or notification is received.

**Acceptance Scenarios**:
1. **Given** SNS configuration, **When** an alert is triggered, **Then** a short summary text MUST be published to the topic.

## Clarifications

### Session 2026-01-18 (Derived from rsudp reference)
- Q: 優先順位と実装対象 → A: Discord, LINE, Google Chat, Amazon SNSを実装する。Telegram, Twitter, Blueskyは対象外。
- Q: Discordの投稿方式 → A: **Incoming Webhook** を使用する。画像は直接アップロード（multipart/form-data）する。
- Q: LINE / Google Chatの画像投稿 → A: **Amazon S3** に `ACL: public-read` でアップロードし、そのパブリックURLを投稿に使用する。
- Q: 投稿のタイミング → A: **Trigger (ALARM)** 時にテキスト通知を送り、**Reset/Snapshot (IMGPATH)** 時に画像付き通知を送る（2段階投稿）。
- Q: LINEの投稿方式 → A: **Messaging API** を使用する。

## Edge Cases

- **Credential Missing**: If SNS credentials are provided but invalid, the system must log an error and continue other pipeline tasks.
- **S3 Upload Timeout**: If S3 upload fails or times out, LINE/Google Chat notifications should be sent as text-only with a warning.
- **Rate Limits**: Handle HTTP 429 responses from Discord/LINE by logging and potentially retrying (though retrying seismic alerts is of limited use if delayed).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Implement an asynchronous `SNSManager` that handles multiple notification providers.
- **FR-002**: **Discord Integration**:
    - Post text on "Trigger".
    - Post rich embed with PNG image on "Reset/Snapshot" using multipart upload.
- **FR-003**: **AWS S3 Integration**:
    - Required for LINE and Google Chat images.
    - MUST upload PNG snapshots to a configured bucket using `public-read` ACL to ensure the platform servers can access the image.
- **FR-004**: **LINE Integration**:
    - Use Messaging API to send push messages to configured User/Group IDs.
    - Post text on "Trigger".
    - Post `ImageMessage` on "Reset/Snapshot" after S3 upload.
- **FR-005**: **Google Chat Integration**:
    - Use Webhooks to post messages.
    - Post text on "Trigger".
    - Post image card/link on "Reset/Snapshot" after S3 upload.
- **FR-006**: **Amazon SNS Integration**:
    - Publish alert text to a specific Topic ARN.
- **FR-007**: All network calls MUST be non-blocking relative to the main data processing pipeline.

### Key Entities

- **NotificationEvent**: Internal structure containing trigger/reset data, intensity, and local image path.
- **S3Client**: Helper for uploading to Amazon S3.
- **WebhookClient**: Generic HTTP client for Discord/Google Chat.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Discord notifications with images are received within 10 seconds of event conclusion.
- **SC-002**: LINE/Google Chat messages contain valid S3 links when image support is enabled.
- **SC-003**: No panics or pipeline delays occur even if all SNS providers are unreachable.
- **SC-004**: Configuration parity with `rsudp`'s SNS sections in `settings.toml`.
