# Implementation Plan: Fix Trigger Parameters for Parity

**Branch**: `025-fix-trigger-params` | **Date**: 2026-01-18 | **Spec**: [/specs/025-fix-trigger-params/spec.md](/specs/025-fix-trigger-params/spec.md)
**Input**: Feature specification from `/specs/025-fix-trigger-params/spec.md`

## Summary

This feature addresses the discrepancy between `rsudp-rust` and the original `rsudp` by implementing dynamic filter coefficient calculation. Instead of using hardcoded coefficients (which ignored config changes), we will implement a Butterworth filter design algorithm (equivalent to `scipy.signal.butter`) directly in Rust. This ensures that the user's `highpass` and `lowpass` settings are correctly applied.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2024)
**Primary Dependencies**: 
- `num-complex` (already in project) for filter pole/zero calculations.
**Storage**: N/A
**Testing**: Unit tests comparing calculated coefficients against known `scipy` outputs.
**Target Platform**: Linux, macOS, Windows.
**Project Type**: Bug Fix / Enhancement.
**Performance Goals**: Initialization cost is negligible; per-sample processing remains O(1).
**Constraints**: Must match `scipy`'s output for `btype='band', output='sos'`.
**Scale/Scope**: Logic contained within `src/trigger.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|-----------|-------|--------|
| I. 安定性と信頼性 | 設定値が正しく反映されることで予測可能性が向上 | ✅ |
| II. 厳密なテスト | `scipy` との係数比較テストを追加 | ✅ |
| III. 高いパフォーマンス | 計算は初期化時のみ | ✅ |
| IV. 明瞭性と保守性 | マジックナンバーを排除し、アルゴリズムを明示 | ✅ |
| V. 日本語による仕様策定 | 済み | ✅ |
| VI. 標準技術スタック | N/A | ✅ |
| VII. 自己検証の義務 | ローカルでの係数確認 | ✅ |
| VIII. ブランチ運用 | フィーチャーブランチを使用している | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/025-fix-trigger-params/
├── plan.md              # This file
├── research.md          # Filter design algorithm details
├── data-model.md        # Updated Filter struct
├── quickstart.md        # Verification steps
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   └── trigger.rs       # Implement filter design logic here
```

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*(No violations detected)*