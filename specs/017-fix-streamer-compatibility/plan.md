# Implementation Plan: Fix Streamer UDP Packet Compatibility for rsudp

**Branch**: `017-fix-streamer-compatibility` | **Date**: 2026-01-18 | **Spec**: [/specs/017-fix-streamer-compatibility/spec.md](/specs/017-fix-streamer-compatibility/spec.md)
**Input**: Feature specification from `/specs/017-fix-streamer-compatibility/spec.md`

## Summary

Modify the `streamer` utility to serialize MiniSEED data into a custom string format that exactly matches `rsudp`'s expected input structure (`{ 'CHANNEL', TIMESTAMP, SAMPLE1, SAMPLE2, ... }`), replacing the current JSON serialization that causes parsing errors.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2024)
**Primary Dependencies**: `serde_json` (will be replaced/bypassed for custom formatting), `chrono` (timestamps)
**Storage**: N/A
**Testing**: `cargo test`, Manual integration test with `rsudp` (Python)
**Target Platform**: Linux, macOS, Windows
**Project Type**: Single (Rust Binary/Library)
**Performance Goals**: Minimal overhead for custom string formatting compared to JSON serialization.
**Constraints**: Must match `rsudp`'s naive string splitting/replacing logic exactly to avoid crashes.
**Scale/Scope**: Affects only the `streamer` binary's packet generation logic.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|-----------|-------|--------|
| I. 安定性と信頼性 | `rsudp`をクラッシュさせないフォーマットを保証するか | ✅ |
| II. 厳密なテスト | 互換性を確認するテストが含まれるか | ✅ |
| III. 高いパフォーマンス | フォーマット変更がパフォーマンスを損なわないか | ✅ |
| IV. 明瞭性と保守性 | カスタムフォーマット生成ロジックが明確か | ✅ |
| V. 日本語による仕様策定 | 仕様書は日本語で書かれているか | ✅ |
| VI. 標準技術スタック | Rust標準機能で実装可能か | ✅ |
| VII. 自己検証の義務 | 開発者が動作確認を行う計画があるか | ✅ |
| VIII. ブランチ運用 | フィーチャーブランチで作業しているか | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/017-fix-streamer-compatibility/
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
│   ├── bin/
│   │   └── streamer.rs  # Target for modification
│   └── parser/          # Potential location for shared formatting logic (if reused)
└── tests/
    └── integration_streamer.rs # Integration tests
```

**Structure Decision**: Modify `rsudp-rust/src/bin/streamer.rs` directly as the logic is specific to this utility's output format.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*(No violations detected)*