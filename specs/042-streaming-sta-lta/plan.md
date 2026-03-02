# Implementation Plan: Streaming STA/LTA Trigger Calculation

**Branch**: `042-streaming-sta-lta` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/042-streaming-sta-lta/spec.md`

## Summary

現在の `trigger.rs` は `add_sample()` 呼び出しごとにバンドパスフィルタ（4段 Biquad）をゼロ状態から再作成し、バッファ全体（3100サンプル）を再フィルタ＆STA/LTA再計算している（Slice モード）。これにより、フィルタ過渡応答が LTA を不自然に押し上げ、ALARM→RESET が 72秒→159.5秒に遅延する。

修正方針: フィルタ状態（s1, s2）と STA/LTA 値を `StaLtaState` に永続化し、1サンプルあたり O(1) でインクリメンタル更新するストリーミング方式に変更する。

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021)
**Primary Dependencies**: `chrono` (timestamps), `serde` (serialization), `tracing` (logging) — all existing
**Storage**: N/A (in-memory per-channel state only)
**Testing**: `cargo test` + Python/ObsPy verification scripts (`verify_stalta.py`)
**Target Platform**: Linux (Raspberry Pi, x86_64)
**Project Type**: Single project (Rust binary)
**Performance Goals**: O(1) per sample (currently O(3100) per sample)
**Constraints**: Fixed-size per-channel state; no growing buffers; no new dependencies
**Scale/Scope**: Single file change (`trigger.rs`) + existing integration test update

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| 原則 | 状態 | 根拠 |
|------|------|------|
| I. 安定性と信頼性 | ✅ Pass | 理論的に正しい連続ストリーミング STA/LTA に変更。rsudp Python/ObsPy の `recursive_sta_lta` と同等の結果を生む |
| II. 厳密なテスト | ✅ Pass | `shindo0.mseed` による Python リファレンスとの比較検証。既存 integration_alert テストの更新 |
| III. 高いパフォーマンス | ✅ Pass | O(3100) → O(1) per sample。約3100倍の計算量削減 |
| IV. コードの明瞭性 | ✅ Pass | バッファ管理コード削除によりシンプル化 |
| V. 日本語仕様 | ✅ Pass | 仕様書は日本語で策定済み |
| VI. 標準技術スタック | ✅ N/A | WebUI 変更なし |
| VII. 自己検証 | ✅ Pass | `shindo0.mseed` + Python 検証スクリプトで動作確認 |
| VIII. ブランチ運用 | ✅ Pass | `042-streaming-sta-lta` ブランチで作業中 |

## Project Structure

### Documentation (this feature)

```text
specs/042-streaming-sta-lta/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 research output
├── data-model.md        # Phase 1 data model output
└── checklists/
    └── requirements.md  # Quality checklist
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   └── trigger.rs           # PRIMARY: Streaming STA/LTA refactoring
└── tests/
    └── integration_alert.rs # UPDATE: Verify trigger behavior post-refactor
```

**Structure Decision**: Single-file refactoring within existing `rsudp-rust/src/trigger.rs`. No new files, modules, or dependencies required. The `add_sample()` public API signature is unchanged; all changes are internal to `StaLtaState` and `TriggerManager`.
