# Implementation Plan: Automated E2E Alert Triggering Test

**Branch**: `020-add-e2e-alert-test` | **Date**: 2026-01-18 | **Spec**: [/specs/020-add-e2e-alert-test/spec.md](/specs/020-add-e2e-alert-test/spec.md)
**Input**: Feature specification from `/specs/020-add-e2e-alert-test/spec.md`

## Summary

Implement an automated end-to-end integration test (`tests/e2e_alert.rs`) that verifies the complete alert triggering pipeline. The test will spawn `rsudp-rust` and `streamer` processes, feed known seismic data, and assert that an alert is logged and an image file is generated.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2024)
**Primary Dependencies**: `std::process::Command` (process spawning), `tempfile` (isolated environments), `regex` (log parsing), `tokio` (async test runner), `port_selector` or equivalent logic (dynamic ports).
**Storage**: Temporary directories for test outputs.
**Testing**: `cargo test --test e2e_alert`.
**Target Platform**: Linux, macOS, Windows.
**Project Type**: Single (Rust Binary/Library).
**Performance Goals**: Test completion < 60s (using high-speed streaming).
**Constraints**: Must cleanly kill subprocesses on failure to avoid zombies.
**Scale/Scope**: New test file only; no changes to production code logic intended.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|-----------|-------|--------|
| I. 安定性と信頼性 | テストが環境を汚染しないか（tempfile使用） | ✅ |
| II. 厳密なテスト | E2Eテストそのものの追加である | ✅ |
| III. 高いパフォーマンス | 高速再生でテスト時間を短縮しているか | ✅ |
| IV. 明瞭性と保守性 | テストコードが独立しており理解しやすいか | ✅ |
| V. 日本語による仕様策定 | 仕様書は日本語で書かれているか | ✅ |
| VI. 標準技術スタック | Rust標準のテスト機能を活用しているか | ✅ |
| VII. 自己検証の義務 | 自動テストにより検証を強化するものである | ✅ |
| VIII. ブランチ運用 | フィーチャーブランチで作業しているか | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/020-add-e2e-alert-test/
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
├── tests/
│   └── e2e_alert.rs     # New E2E test file
├── Cargo.toml           # Add dev-dependencies (regex, port_selector?)
```

**Structure Decision**: Add a new integration test file `rsudp-rust/tests/e2e_alert.rs`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*(No violations detected)*