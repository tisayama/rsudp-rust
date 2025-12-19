# Feature Specification: STA/LTA Calculation Logic

**Feature Branch**: `003-sta-lta-calc`
**Created**: 2025-12-19
**Status**: Draft
**Input**: User description: "rsudpと同様のSTA/LTAの計算ができるように、データ処理ロジックを作成してください。同じ結果が出ることを担保できるようなテストスクリプトも作ってください。"

## Clarifications

### Session 2025-12-19

- Q: 具体的にどのバリアントのSTA/LTAアルゴリズムを実装すべきですか？ → A: 再帰的 STA/LTA (Recursive)
- Q: この計算モジュールへの入力データ形式はどうあるべきですか？ → A: パース済み数値データ (Vec<f64>)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Calculate STA/LTA (Priority: P1)

As a data scientist, I want a Rust implementation of the STA/LTA algorithm that produces results identical to the standard Python implementation (`rsudp`/`obspy`), so that I can reliably detect seismic events with high performance.

**Why this priority**: This is the core algorithmic component for event detection.

**Independent Test**: Create a unit test that feeds a predefined synthetic waveform (e.g., a step function or a sine wave with a spike) to both the Rust implementation and a Python reference script, asserting that the output values match within a small floating-point tolerance.

**Acceptance Scenarios**:

1.  **Given** a stream of seismic amplitude data (floating point numbers), **When** the STA/LTA function is applied, **Then** it produces a stream of ratio values.
2.  **Given** a specific input dataset used in `rsudp` tests, **When** processed by this Rust module, **Then** the output matches the `rsudp` output exactly (or within 1e-6 tolerance).

---

### Edge Cases

- What happens when the input stream is shorter than the LTA window? (Should likely return 0 or wait)
- How does the system handle `NaN` or `Inf` in input data?
- What happens at the very beginning of the stream (initialization transient)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST implement the recursive STA/LTA algorithm (`recursive_sta_lta` equivalent).
- **FR-002**: The implementation MUST allow configuration of STA window length and LTA window length (in samples).
- **FR-003**: The system MUST accept parsed numerical waveform data as input (e.g., `Vec<f64>`).
- **FR-004**: The system MUST provide a Python-compatible test script to verify correctness against `obspy`/`rsudp`.

### Key Entities *(include if feature involves data)*

- **StaLtaFilter**: A struct holding the state (current STA, current LTA, coefficients) for continuous processing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The calculated STA/LTA ratio matches the `obspy.signal.trigger.recursive_sta_lta` output with a maximum error of 1e-6 for a test dataset of 10,000 samples.
- **SC-002**: The Rust implementation processes 1 hour of 100Hz data in under 100ms (high performance).