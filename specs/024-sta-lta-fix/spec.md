# Feature Specification: Fix STA/LTA Trigger Behavior

**Feature Branch**: `024-sta-lta-fix`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "STA/LTAのアラート機能のトリガーの挙動が、rsudpのオリジナル実装と大きく異なるようです。以下のパラメーターでテストランしてたしかめていますが、同じタイミングでは発生しなかったり、あなたの実装では10分実行していると何度も何度もアラートが出たりします...同じ動きをするようにrust実装を変更してください。"

## Clarifications

### Session 2026-01-18
- Q: 再帰的STA/LTAのアルゴリズム → A: `obspy.signal.trigger.recursive_sta_lta` (Standard: `c = 1.0 / n_samples`) を使用する。
- Q: ウォームアップ期間の扱い → A: `rsudp` は LTA 期間分のパケット (`wait_pkts`) を読み捨てることでウォームアップを行っているため、これを模倣する。
- Q: バンドパスフィルターの仕様 → A: `c_alert.py` では `stream.filter()` に `corners` を指定していないため、Obspy のデフォルトである 4 (cascaded 2nd order) が使用されている。
- Q: トリガー判定のデバウンス → A: `duration` パラメータが 0 の場合はデバウンスなし。0 より大きい場合は、閾値超過が `duration` 秒間継続した後にトリガーするロジックが存在する。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Parity with Python Implementation (Priority: P1)

As a user, I want the Rust implementation of the STA/LTA trigger to behave exactly like the original Python `rsudp` implementation when given the same input data and configuration, so that I can rely on the system for accurate seismic event detection without false positives or missed events.

**Why this priority**: The current Rust implementation is reportedly unstable (frequent false positives) or inconsistent compared to the reference implementation, undermining the core value of the tool.

**Independent Test**:
1. Use `streamer` to replay a known MiniSEED file (`fdsnws.mseed`) to both the Python `rsudp` (reference) and `rsudp-rust` (target).
2. Configure both with identical STA/LTA parameters (STA=6s, LTA=30s, Threshold=1.1, Reset=0.5, Bandpass=0.1-2.0Hz).
3. Compare the trigger start times and durations. They should match within a reasonable margin of error (e.g., < 1s).

**Acceptance Scenarios**:

1. **Given** a 10-minute seismic data stream with no significant events, **When** processed with the specified high-sensitivity parameters (Threshold 1.1), **Then** the system MUST NOT trigger continuous/flapping alerts if the reference implementation does not.
2. **Given** a seismic event in the data stream, **When** processed, **Then** the Trigger and Reset times MUST align with the Python implementation's behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The STA/LTA calculation logic MUST match `obspy.signal.trigger.recursive_sta_lta` (used by `rsudp`) exactly.
- **FR-002**: The bandpass filter implementation MUST use a 4th order (2nd order cascaded) Butterworth filter, matching `obspy`'s default behavior when `corners` is not specified.
- **FR-003**: The system MUST discard the first `LTA` seconds of data (warm-up period) before attempting any trigger detection, matching `rsudp`'s `wait_pkts` logic.
- **FR-004**: The system MUST support a `duration` parameter. If `duration > 0`, the system MUST only trigger if the threshold is exceeded continuously for `duration` seconds. If `duration == 0`, it triggers immediately (no debounce).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero false positive triggers in a 10-minute background noise test (relative to reference implementation).
- **SC-002**: Trigger start times match `rsudp` reference output within ±0.5 seconds for standard test events.