# Implementation Plan: Fix UDP Packet Granularity for rsudp Compatibility

**Branch**: `018-fix-packet-granularity` | **Date**: 2026-01-18 | **Spec**: [/specs/018-fix-packet-granularity/spec.md](/specs/018-fix-packet-granularity/spec.md)
**Input**: Feature specification from `/specs/018-fix-packet-granularity/spec.md`

## Summary

Optimize the `streamer` utility to split large MiniSEED records into smaller UDP packets (default 25 samples) and transmit them at intervals matching the sample duration (e.g., 0.25s for 100Hz). This mimics real hardware behavior and prevents visual artifacts in `rsudp`.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2024)
**Primary Dependencies**: `tokio` (async/sleep), `chrono` (timestamps), `clap` (CLI args)
**Storage**: N/A
**Testing**: `cargo test`, manual verification with `rsudp`
**Target Platform**: Linux, macOS, Windows
**Project Type**: Single (Rust Binary/Library)
**Performance Goals**: Accurate sub-second sleep timing for smooth streaming.
**Constraints**: Must maintain timestamp continuity across packet boundaries.
**Scale/Scope**: Logic contained within `rsudp-rust/src/bin/streamer.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|-----------|-------|--------|
| I. 安定性と信頼性 | 分割によるデータ欠損がないか | ✅ |
| II. 厳密なテスト | パケット分割とタイミングのテストが含まれるか | ✅ |
| III. 高いパフォーマンス | 非同期スリープで効率的に実装されるか | ✅ |
| IV. 明瞭性と保守性 | 分割ロジックが明確か | ✅ |
| V. 日本語による仕様策定 | 仕様書は日本語で書かれているか | ✅ |
| VI. 標準技術スタック | Rust標準 + Tokioで実装可能か | ✅ |
| VII. 自己検証の義務 | 開発者が動作確認を行う計画があるか | ✅ |
| VIII. ブランチ運用 | フィーチャーブランチで作業しているか | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/018-fix-packet-granularity/
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
└── tests/
    └── integration_streamer.rs # Integration tests
```

**Structure Decision**: Logic resides entirely in `streamer.rs`. No new modules required.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*(No violations detected)*