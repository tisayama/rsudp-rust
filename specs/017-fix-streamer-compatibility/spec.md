# Feature Specification: Fix Streamer UDP Packet Compatibility for rsudp

**Feature Branch**: `017-fix-streamer-compatibility`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "streamerが発生するUDPパケットがrsudpの期待しているフォーマットと違うようです。われわれの目標はrsudpへの完全互換ですので。streamerの互換性を向上させてください。 具体的にはstreamerでmseedを読み込んでUDPパケットをrsudpに送信したところ、すぐに以下のようなExceptionが発生しました。 (Error log details regarding 'invalid literal for int() with base 10: '16470]')"

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2026-01-18
- Q: 正確なUDPパケットフォーマット → A: `rsudp` のソースコードを調査し、期待される正確な文字列フォーマットを特定する
- Q: タイムスタンプの精度と形式 → A: Unix Epoch からの秒数を示す浮動小数点数 (decimal seconds)
- Q: チャンネル名のクォート処理 → A: シングルクォートで囲む (`'EHZ'`)

### User Story 1 - Streamer UDP Packet Compatibility (Priority: P1)

As a system developer, I want the `streamer` utility to send UDP packets in a format that is fully compatible with `rsudp` so that I can use the Rust-based streamer to test or feed data to the original Python-based `rsudp` implementation without causing exceptions.

**Why this priority**: The reported exception (`ValueError: invalid literal for int() with base 10`) indicates a breaking format mismatch. `rsudp` is the reference implementation, and `streamer` must conform to its expected input format to be a valid tool in this ecosystem.

**Independent Test**:
1. Run the original `rsudp` application (Python) listening on a local port (e.g., 8888).
2. Run the Rust-based `streamer` targeting that port.
3. Verify that `rsudp` receives and processes data without crashing or logging format errors.

**Acceptance Scenarios**:

1. **Given** `rsudp` is running and listening for UDP packets, **When** `streamer` sends a packet derived from an mseed record, **Then** `rsudp` successfully parses the packet without raising a `ValueError`.
2. **Given** the current failure mode (`16470]`), **When** the fix is applied, **Then** the trailing bracket `]` (or other formatting artifacts) must be correctly formatted or removed from the data payload string sent over UDP.

### Edge Cases

- **Variable Sample Rates**: Ensure the format holds for different sampling rates if that affects packet structure.
- **Packet Size**: Ensure packets remain within standard MTU limits if formatting changes increase size significantly (unlikely here, but good to note).
- **Data Types**: Verify handling of negative integers vs. positive integers in the sample list string representation.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `streamer` MUST serialize MiniSEED record data into a string format that exactly matches `rsudp`'s non-standard parsing logic.
    - **Format**: `{ 'CHANNEL', TIMESTAMP, SAMPLE1, SAMPLE2, ... }`
    - The string MUST start with a character that `getCHN` skips (like `{`) and end with a character that `getSTREAM` removes (like `}`).
    - **Channel**: MUST be wrapped in single quotes (e.g., `'EHZ'`).
    - **Timestamp**: MUST be a float representing seconds since Unix Epoch.
    - **Samples**: MUST be integers separated by commas.
- **FR-002**: The `streamer` MUST NOT use standard JSON array brackets `[` or `]` if it prevents `rsudp` from parsing integers correctly (as evidenced by the `16470]` error).
- **FR-003**: The `streamer` MUST ensure no extraneous spaces or characters interfere with the `split(',')` and `int()` conversion steps.

### Key Entities

- **UDPPacket**: The payload sent over the network. Should roughly correspond to a structure like `{'channel': 'EHZ', 'timestamp': 1234567890.123, 'data': [1, 2, 3, ...]}` but formatted specifically for `rsudp`'s parser.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `rsudp` runs for at least 10 minutes receiving data from `streamer` without a single `ValueError` or crash.
- **SC-002**: The data plotted or processed by `rsudp` matches the input MiniSEED data values (verified by visual inspection or log comparison).