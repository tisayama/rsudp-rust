# Feature Specification: Fix UDP Packet Granularity for rsudp Compatibility

**Feature Branch**: `018-fix-packet-granularity`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "前任者がrsudpは25サンプルが1パケットにまとまっていると短絡的に結論を出してしまっていたが、もう少し注意深くrsudp実装やテストデータを調査し、1パケットあたりのサンプル数、送信間隔を模倣してほしい。"

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2026-01-18
- Q: 1パケットあたりのサンプル数 → A: テストデータの調査により **25サンプル** であることを確認。ユーザー設定で変更可能（デフォルト25）とする。
- Q: 送信間隔の制御 → A: サンプル数とレートから算出した持続時間分（例: 25/100Hz = 0.25s）、パケットごとに正確にスリープする。

### User Story 1 - Optimize Packet Granularity (Priority: P1)

As a system developer, I want the `streamer` utility to send UDP packets with a sample count and transmission frequency that accurately mimics the real Raspberry Shake hardware (or standard rsudp behavior), so that the receiving `rsudp` application can plot data smoothly without artifacts like "banding" or stuttering.

**Why this priority**: The current implementation sends entire MiniSEED records (often ~500 samples) at once, causing bursty updates and visual artifacts in `rsudp`'s real-time plot. Correct granularity is essential for a faithful simulation.

**Independent Test**:
1. Run `rsudp` (Python) listening on a local port.
2. Run the updated `streamer` targeting that port.
3. Observe the `rsudp` plot. It should scroll smoothly, similar to real device data, without periodic gaps or sudden jumps (banding).
4. Verify via packet capture or logs that packets are sent frequently (e.g., ~4 times per second for 100Hz/25 samples) rather than once every few seconds.

**Acceptance Scenarios**:

1. **Given** a 100Hz MiniSEED file, **When** streamed at 1x speed, **Then** `streamer` sends approximately 4 packets per second, each containing ~25 samples (or the discovered optimal count).
2. **Given** the visual output in `rsudp`, **When** using the updated streamer, **Then** the scrolling waveform appears continuous and smooth, eliminating the "striped" pattern caused by bursty data.

### Edge Cases

- **High Sample Rates**: Ensure the system can keep up with packet generation for higher sample rates (e.g., 200Hz, 500Hz) if applicable.
- **Packet Overhead**: More small packets mean higher UDP overhead. Ensure this doesn't saturate the local network stack (unlikely for typical seismic data rates).
- **Processing Lag**: Ensure the `streamer` logic doesn't drift significantly from real-time due to the overhead of smaller sleep intervals.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `streamer` MUST split MiniSEED data into small chunks. The default chunk size MUST be **25 samples** per packet (mimicking `rsudp` test data).
- **FR-002**: The `streamer` SHOULD allow the user to override the samples-per-packet count via a CLI argument (e.g., `--samples-per-packet`).
- **FR-003**: The `streamer` MUST calculate the transmission interval for each packet as: `interval = samples_in_packet / sample_rate`.
- **FR-004**: The `streamer` MUST sleep for the calculated `interval` after sending each packet to maintain real-time parity.
- **FR-005**: The `streamer` MUST calculate precise timestamps for each chunk (start of the chunk), ensuring continuous time across record boundaries.

### Key Entities

- **PacketChunk**: A subset of a MiniSEED record containing a specific number of samples and a calculated start time.
- **TransmissionLoop**: The logic controlling the precise timing of UDP sends.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Packet transmission frequency matches the expected rate (e.g., ~4Hz for 100Hz/25samples) within a 10% margin.
- **SC-002**: `rsudp` plot smoothness is visually indistinguishable from a real device connection (subjective but verifiable comparison).
- **SC-003**: No data loss or significant time drift over a 10-minute stream session.