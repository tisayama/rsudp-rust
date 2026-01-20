# Implementation Plan: Fix STA/LTA Trigger Behavior

**Branch**: `024-sta-lta-fix` | **Date**: 2026-01-18 | **Spec**: [/specs/024-sta-lta-fix/spec.md](/specs/024-sta-lta-fix/spec.md)
**Input**: Feature specification from `/specs/024-sta-lta-fix/spec.md`

## Summary

Align the STA/LTA trigger implementation with `rsudp`'s original Python code to resolve instability and false positives. This involves updating the bandpass filter to 4th order, enforcing a strict warm-up period (dropping data), and implementing a duration-based debounce timer.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2024)
**Primary Dependencies**: None (Standard Library + internal logic)
**Storage**: In-memory state.
**Testing**: Unit tests for filter and trigger logic; manual verification via `streamer`.
**Target Platform**: Linux, macOS, Windows.
**Project Type**: Bug Fix / Refactor.
**Performance Goals**: Minimal overhead added by higher order filter.
**Constraints**: Must match `obspy`/`rsudp` behavior exactly.
**Scale/Scope**: Logic contained within `src/trigger.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|-----------|-------|--------|
| I. 安定性と信頼性 | 誤検知の排除により信頼性が向上する | ✅ |
| II. 厳密なテスト | `rsudp` との比較検証を行う | ✅ |
| III. 高いパフォーマンス | 計算コストは微増だが許容範囲内 | ✅ |
| IV. 明瞭性と保守性 | ロジックを `rsudp` に合わせることで仕様が明確化 | ✅ |
| V. 日本語による仕様策定 | 済み | ✅ |
| VI. 標準技術スタック | N/A | ✅ |
| VII. 自己検証の義務 | ローカルでの再現テストを行う | ✅ |
| VIII. ブランチ運用 | フィーチャーブランチを使用している | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/024-sta-lta-fix/
├── plan.md              # This file
├── research.md          # Analysis of rsudp logic
├── data-model.md        # Updated State struct
├── quickstart.md        # Verification steps
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   └── trigger.rs       # Core logic changes
```

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*(No violations detected)*