# Implementation Plan: Restore STA/LTA Alerts Functionality

**Branch**: `019-restore-sta-lta-alert` | **Date**: 2026-01-18 | **Spec**: [/specs/019-restore-sta-lta-alert/spec.md](/specs/019-restore-sta-lta-alert/spec.md)
**Input**: Feature specification from `/specs/019-restore-sta-lta-alert/spec.md`

## Summary

Investigate and fix the regression in STA/LTA alert triggering caused by recent changes to the streamer (small packet chunking). The primary focus is ensuring `TriggerManager` and its filters correctly maintain state across fragmented data streams.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2024)
**Primary Dependencies**: `rsudp-rust` internal modules (`trigger`, `pipeline`)
**Storage**: N/A
**Testing**: `cargo test`, manual integration with `streamer`
**Target Platform**: Linux, macOS, Windows
**Project Type**: Single (Rust Binary/Library)
**Performance Goals**: Minimal latency in triggering logic.
**Constraints**: Must handle arbitrary packet sizes (25 samples or more/less).
**Scale/Scope**: Logic contained within `rsudp-rust/src/trigger.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|-----------|-------|--------|
| I. 安定性と信頼性 | アラート漏れがないことを保証するか | ✅ |
| II. 厳密なテスト | 分割データのテストケースが含まれるか | ✅ |
| III. 高いパフォーマンス | 修正が処理速度を低下させないか | ✅ |
| IV. 明瞭性と保守性 | 状態管理のロジックが明確か | ✅ |
| V. 日本語による仕様策定 | 仕様書は日本語で書かれているか | ✅ |
| VI. 標準技術スタック | Rust標準機能で実装可能か | ✅ |
| VII. 自己検証の義務 | 開発者が動作確認を行う計画があるか | ✅ |
| VIII. ブランチ運用 | フィーチャーブランチで作業しているか | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/019-restore-sta-lta-alert/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── trigger.rs       # Target for modification (Filter state)
│   └── pipeline.rs      # Verification of data passing
└── tests/
    └── integration_alert.rs # Integration tests
```

**Structure Decision**: Logic resides in `trigger.rs`. `pipeline.rs` needs review but likely no changes if `trigger` is fixed.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*(No violations detected)*