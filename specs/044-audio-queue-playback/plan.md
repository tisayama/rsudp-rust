# Implementation Plan: アラート音声の順序再生（キューイング）

**Branch**: `044-audio-queue-playback` | **Date**: 2026-03-09 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/044-audio-queue-playback/spec.md`

## Summary

現在の `AudioManager` は状態を持たず、パイプラインが `tokio::task::spawn_blocking` で毎回独立スレッドを起動して `play_file()` を呼ぶ。これを FIFO キューイング方式に変更する。`AudioManager::new()` で専用の再生スレッドと `std::sync::mpsc` チャネルを起動し、`queue_file()` メソッドでファイルパスをチャネルに送信する。再生スレッドはチャネルから順番にファイルパスを受信し、既存の `play_file()` ロジック（再生ごとに ALSA ストリームを新規作成）で1件ずつ再生する。

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021)
**Primary Dependencies**: `rodio` 0.17 (symphonia-mp3, symphonia-wav), `tracing` (ログ), `std::sync::mpsc` (チャネル)
**Storage**: N/A
**Testing**: `cargo test` — 単体テスト（`sound.rs` 内）
**Target Platform**: Linux (Raspberry Pi / x86_64)
**Project Type**: Single project (既存モジュール改修)
**Performance Goals**: キュー空状態での再生開始遅延 < 100ms
**Constraints**: 再生ごとに ALSA OutputStream を新規作成（Raspberry Pi 長時間稼働対策）
**Scale/Scope**: 変更ファイル2つ (`sound.rs`, `pipeline.rs`)、新規依存なし

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| 原則 | 判定 | 備考 |
|------|------|------|
| I. 安定性と信頼性 | PASS | FIFO キューにより音声重複を排除、ALSA fresh stream 設計を維持 |
| II. 厳密なテスト | PASS | `queue_file` のキューイング動作を単体テストで検証 |
| III. 高いパフォーマンス | PASS | mpsc チャネル送信は O(1)、パイプラインをブロックしない |
| IV. コードの明瞭性と保守性 | PASS | 既存 `play_file` ロジック維持、キューイングは標準パターン |
| V. 日本語による仕様策定 | PASS | 仕様書・計画書は日本語 |
| VI. 標準技術スタック | N/A | WebUI 変更なし |
| VII. 自己検証の義務 | PASS | ビルド・テスト・clippy で検証 |
| VIII. ブランチ運用 | PASS | フィーチャーブランチ `044-audio-queue-playback` で作業 |

## Project Structure

### Documentation (this feature)

```text
specs/044-audio-queue-playback/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
rsudp-rust/src/
├── sound.rs             # AudioManager 改修: mpsc チャネル + 再生スレッド追加
└── pipeline.rs          # 呼び出し側変更: spawn_blocking → queue_file()
```

**Structure Decision**: 既存ファイル2つの改修のみ。新規ファイル不要。

## Complexity Tracking

該当なし — 憲法違反なし。
