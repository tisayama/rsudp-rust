# Feature Specification: RSUDP Test Run and Comparison

**Feature Branch**: `026-rsudp-test-run`
**Created**: 2026-01-18
**Status**: Draft
**Input**: User description: "前任者にお願いしていたSTA/LTAのアラート機能のトリガーがうまくいっておらず、rsudpのオリジナル実装と異なるようです...以下のパラメーターでテストランしてたしかめていますが...rsudp実装をテストランして、rust実装もテストランして入念に比較してほしいです。"

## Clarifications

### Session 2026-01-18
- Q: Python環境のセットアップ → A: プロジェクト内に `venv` を作成し、必要な依存ライブラリをインストールして実行する。
- Q: 比較レポートの形式と許容誤差 → A: 詳細CSVレポートを出力し、許容誤差は ±0.5秒とする。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run Python rsudp Reference (Priority: P1)

As a developer, I want to run the original Python `rsudp` implementation with a specific configuration and dataset so that I can establish a baseline for "correct" behavior (trigger timing, duration, reset).

**Why this priority**: Without a ground-truth execution log from the Python implementation, we cannot definitively say whether the Rust implementation is buggy or correct.

**Independent Test**:
1. Configure `references/rsudp/rsudp/settings.json` (or equivalent) with the user-provided parameters.
2. Run `references/rsudp/unix-start-rsudp.sh` (or direct python command) using `references/mseed/fdsnws.mseed` as input.
3. Capture the stdout logs to a file (`rsudp_python.log`).
4. Identify timestamps of "Trigger" and "Reset" events.

**Acceptance Scenarios**:

1. **Given** the user's config and test data, **When** rsudp is run, **Then** it produces a log file containing trigger events (or lack thereof).

### User Story 2 - Run Rust rsudp-rust Target (Priority: P2)

As a developer, I want to run `rsudp-rust` with the *exact same* configuration and dataset so that I can compare its output against the Python baseline.

**Why this priority**: Comparing "apples to apples" is essential for debugging signal processing logic.

**Independent Test**:
1. Configure `rsudp-rust/settings.toml` with the user-provided parameters.
2. Run `rsudp-rust` using `references/mseed/fdsnws.mseed` (via `streamer` or file input mode).
3. Capture stdout logs to a file (`rsudp_rust.log`).
4. Identify timestamps of "Trigger" and "Reset" events.

**Acceptance Scenarios**:

1. **Given** the same config and data, **When** rsudp-rust is run, **Then** it produces a log file containing trigger events.

### User Story 3 - Comparative Analysis (Priority: P3)

As a developer, I want to compare the two logs side-by-side to identify discrepancies in trigger start time, duration, and max ratio.

**Why this priority**: This analysis will pinpoint whether the issue is sensitivity (threshold too low/high), filtering (signal content different), or logic (state machine differences).

**Independent Test**:
1. Create a comparison report listing events from both logs.
2. Calculate delta = (Rust Time - Python Time).

**Acceptance Scenarios**:

1. **Given** both logs, **When** compared, **Then** a report is generated showing the specific discrepancies (e.g., "Rust triggered 2s early", "Rust triggered 5 times vs Python's 1 time").

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST set up a dedicated Python virtual environment (`venv`) and install `rsudp` dependencies to run the reference implementation without polluting the host system.
- **FR-002**: The system MUST support running `rsudp-rust` with an identical configuration to the Python run.
- **FR-003**: The input dataset MUST be consistent (same MiniSEED file) for both runs.
- **FR-004**: The system MUST parse the output logs from both runs and generate a CSV comparison report (`comparison_report.csv`) listing matched events and their time differences.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Execution of Python `rsudp` completes without error and produces a log.
- **SC-002**: Execution of `rsudp-rust` completes without error and produces a log.
- **SC-003**: A comparison report is generated detailing the differences.