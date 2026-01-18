# Feature Specification: Restore STA/LTA Alerts Functionality

**Feature Branch**: `019-restore-sta-lta-alert`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "streamerからrustrsudpにstreamしても、うまくalertが発生しないため、以前のようにSTA/LTAベースのアラートが処理できるようにしてほしいです。streamer自体の動作はrsudpと組み合わせて確認しましたが、問題なかったのでrustrsudp単体の問題のようです。"

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2026-01-18
- Q: アラート不発の原因に関する仮説 → A: STA/LTAの計算ロジック自体が、小さなチャンク（25サンプル）に対応できていない（バッファリング不足など）
- Q: トリガーロジックの修正方針 → A: フィルタの内部状態が呼び出し間で維持されているか確認し、必要なら修正する

### User Story 1 - Restore STA/LTA Alert Triggering (Priority: P1)

As a system operator, I want `rsudp-rust` to correctly trigger alerts based on the STA/LTA algorithm when receiving data from `streamer` (or any MiniSEED source), so that I can detect seismic events as reliably as I could in previous versions.

**Why this priority**: The core value proposition of the system is event detection. Currently, despite valid data streaming, alerts are not firing, which is a regression or a side effect of recent changes (likely related to timestamp handling or packet granularity fixes).

**Independent Test**:
1. Start `rsudp-rust` listening on a UDP port (e.g., 8888).
2. Start `streamer` sending a known event dataset (e.g., `fdsnws.mseed` which contains a large quake) to that port.
3. Observe `rsudp-rust` logs for "Triggered" messages or check the `alerts/` directory for generated event files.

**Acceptance Scenarios**:

1. **Given** `rsudp-rust` is running with default trigger settings, **When** `streamer` sends the `fdsnws.mseed` data at 1x speed, **Then** `rsudp-rust` logs a "Triggered" event message corresponding to the quake in the file.
2. **Given** the trigger activation, **When** the event concludes, **Then** an alert snapshot (image) is generated in the configured output directory.

### Edge Cases

- **Packet Granularity Impact**: Ensure the recent fix for packet chunking (25 samples/packet) hasn't negatively impacted the `TriggerManager`'s buffer filling or window calculations (e.g., if it expects larger chunks or different timing).
- **Timestamp Precision**: Verify if `TriggerManager` handles the high-precision timestamps from the new `streamer` correctly without drifting or resetting too aggressively.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `TriggerManager` MUST correctly accumulate incoming sample streams that are chunked into small packets (e.g., 25 samples) into its STA/LTA buffers without premature calculation or state reset.
    - **Action**: Verify if the internal buffer or filter state updates correctly when samples arrive in small batches versus large records.
- **FR-002**: The system MUST detect STA/LTA ratio exceedance events from the input stream.
- **FR-003**: The `pipeline` logic MUST properly pass the absolute timestamp of each sample (or chunk) to the `TriggerManager` to maintain time continuity.
- **FR-004**: The system MUST NOT mistakenly reset the trigger state due to minor jitter in packet arrival times (related to the previous "temporal jump" fix).
- **FR-005**: The `TriggerManager`'s `BandpassFilter` MUST maintain its internal state (`x1`, `x2`, `y1`, `y2`) correctly across multiple `add_sample` calls to ensure continuous filtering regardless of input chunk size.

### Key Entities

- **TriggerManager**: The logic component calculating STA/LTA ratios and state.
- **Pipeline**: The orchestrator moving data from `Receiver` to `TriggerManager`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Running the standard test dataset (`fdsnws.mseed`) triggers exactly one major alert event (matching the known quake).
- **SC-002**: The trigger time recorded matches the event time in the MiniSEED file within +/- 1 second.
- **SC-003**: An alert image is successfully generated and saved.