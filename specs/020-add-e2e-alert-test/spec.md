# Feature Specification: Automated E2E Alert Triggering Test

**Feature Branch**: `020-add-e2e-alert-test`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "現在の手動検証（streamerを起動してログを目視確認）を自動化し、リグレッションを防止するための統合テストスイートを追加したいです。..."

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2026-01-18
- Q: テスト時のポート番号選択 → A: 空いているポートを動的に取得する（例: ポート0番でバインドしてOSから割り当てを受ける）
- Q: アラート発生の判定基準 → A: ログメッセージ（ALARM）と画像ファイル（PNG）の生成の両方を確認する
- Q: テスト用バイナリの扱い → A: `cargo run --bin ...` を `Command` で呼び出す（または `assert_cmd` などのツールを使用する）

### User Story 1 - E2E Integration Test for Alert Triggering (Priority: P1)

As a developer, I want an automated integration test that spins up both `rsudp-rust` and `streamer`, feeds known seismic data, and asserts that an alert is triggered, so that I can prevent regressions in the alert logic or packet handling without manual verification.

**Why this priority**: Manual verification is error-prone and time-consuming. Recent regressions in packet compatibility and timestamp handling highlight the need for a robust, automated end-to-end test.

**Independent Test**:
Run `cargo test --test integration_e2e_alert` (or similar). The test should compile, execute both binaries (or library equivalents), and pass only if the "ALARM" condition is met within a specified timeout.

**Acceptance Scenarios**:

1. **Given** the `fdsnws.mseed` dataset (containing a known quake), **When** the test runs `rsudp-rust` (receiver) and `streamer` (sender) at high speed (e.g., 100x), **Then** the test MUST detect an "ALARM" log message or a generated alert file from `rsudp-rust`.
2. **Given** the test environment, **When** the test completes (success or failure), **Then** all spawned subprocesses (`rsudp-rust`, `streamer`) MUST be terminated cleanly.
3. **Given** a regression (e.g., broken parser), **When** the test runs, **Then** it MUST fail with a clear timeout or error message indicating no alert was triggered.

### Edge Cases

- **Port Conflicts**: The test MUST use a dynamically assigned UDP port (e.g., by binding to port 0) to avoid conflicts with other processes or parallel test runs.
- **Test Duration**: The test must balance speed (100x) with reliability (not overloading the pipeline). A timeout of ~30-60 seconds is reasonable.
- **Log Buffering**: Ensure the test captures output in real-time or flushes buffers so `grep` (or equivalent assertion logic) doesn't hang.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a new integration test file (e.g., `tests/e2e_alert.rs`) that acts as the test harness.
- **FR-002**: The test harness MUST spawn `rsudp-rust` as a background process using `cargo run --bin rsudp-rust` (or equivalent) listening on a dynamically allocated local UDP port.
- **FR-003**: The test harness MUST spawn `streamer` as a background process using `cargo run --bin streamer` (or equivalent) sending `fdsnws.mseed` data to that port at high speed (e.g., `--speed 100.0`).
- **FR-004**: The test harness MUST capture the standard output/error of `rsudp-rust`.
- **FR-005**: The test harness MUST assert that the string "ALARM" (or specific trigger message) appears in the output AND that a corresponding alert image file is created in the output directory within a defined timeout (e.g., 60 seconds).
- **FR-006**: The test harness MUST strictly cleanup (kill) all spawned processes upon test completion or panic.

### Key Entities

- **TestHarness**: Rust code responsible for process management and assertion.
- **Subprocess**: `std::process::Child` wrappers for the binaries.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The new test `cargo test --test e2e_alert` passes consistently in the CI/local environment using the provided `fdsnws.mseed`.
- **SC-002**: The test completes in under 60 seconds (target ~10-20 seconds with high-speed streaming).
- **SC-003**: Introducing a known bug (e.g., breaking the parser) causes the test to fail.