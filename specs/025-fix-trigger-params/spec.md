# Feature Specification: Fix Trigger Parameters for Parity

**Feature Branch**: `025-fix-trigger-params`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "前任者にお願いしていたSTA/LTAのアラート機能のトリガーがうまくいっておらず、rsudpのオリジナル実装と異なるようです...以下のパラメーターでテストランしてたしかめていますが、同じタイミングでは発生しなかったり、あなたの実装では10分実行していると何度も何度もアラートが出たりします..."

## Clarifications

### Session 2026-01-18
- Q: テスト用データセット → A: `references/mseed/fdsnws.mseed` を使用する。
- Q: フィルタ係数の計算方法 → A: Rust内で係数計算を実装し、設定値 (`highpass`, `lowpass`) を動的に反映させる。
- Q: 係数計算の実装手段 → A: 外部クレートを使わず、Butterworthフィルタの計算ロジックを自前で実装して `scipy` との完全互換を目指す。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify and Fix Trigger Parameters (Priority: P1)

As a user, I want the Rust implementation to accept the same configuration parameters as `rsudp` and behave identically, specifically ensuring that `duration`, `sta`, `lta`, `threshold`, and `reset` parameters are applied correctly to prevent flapping and missed events.

**Why this priority**: Even with the logic fixes in feature 024, the user reports continued discrepancies. This suggests a potential issue with how parameters are passed, parsed, or applied (e.g., unit conversions, default overrides).

**Independent Test**:
1. Configure `settings.toml` with the exact JSON provided by the user.
2. Run `rsudp` (Python) and `rsudp-rust` against the same 10-minute dataset.
3. Verify that `rsudp-rust` does not trigger repeatedly (flapping) and triggers at the same timestamp as `rsudp`.

**Acceptance Scenarios**:

1. **Given** the user's specific high-sensitivity config (Threshold 1.1, Reset 0.5), **When** running for 10 minutes, **Then** the number of triggers MUST match the reference implementation exactly.
2. **Given** `duration=0.0`, **When** a short spike occurs, **Then** a trigger SHOULD occur immediately (if valid), matching reference behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST verify that `sta` and `lta` seconds are correctly converted to sample counts based on the actual sampling rate (100Hz).
- **FR-002**: The system MUST dynamically calculate 4th-order Butterworth bandpass filter coefficients (SOS format) using a self-implemented algorithm equivalent to `scipy.signal.butter`, ensuring `highpass` and `lowpass` settings are correctly applied.
- **FR-003**: The system MUST verify that the `deconvolve` parameter is respected (though likely false in this test case).
- **FR-004**: The system MUST provide a way to dump/log the active internal configuration at startup to confirm parameters are not being silently overridden or defaulted.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Trigger count and timestamps match `rsudp` reference output for the provided test case.
- **SC-002**: Startup logs confirm that all parameters (STA, LTA, Threshold, Reset, Duration, Filters) match the input configuration exactly.