# Implementation Plan: UDP Data Forwarding

**Branch**: `034-udp-forward` | **Date**: 2026-02-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/034-udp-forward/spec.md`

## Summary

受信した地震データUDPパケットを、設定で指定された1つ以上のリモート宛先にフォワーディングする機能を実装する。Python rsudpの`c_forward.py`に相当するRust実装。チャンネルフィルタリング、データ/アラームの独立制御、ランタイムログベースの統計モニタリングを含む。パイプライン内で`parse_any()`後の生バイトを非同期タスク経由で転送し、メインパイプラインをブロックしない設計。

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021)
**Primary Dependencies**: `tokio` (async runtime, `UdpSocket`, `mpsc`), `tracing` (logging), `serde` (config deserialization) — すべて既存依存
**Storage**: N/A (インメモリ統計のみ)
**Testing**: `cargo test` — ユニットテスト + 統合テスト（ローカルUDPリスナー使用）
**Target Platform**: Linux (x86_64, ARM/Raspberry Pi)
**Project Type**: Single project (既存 `rsudp-rust` Cargo workspace)
**Performance Goals**: パイプライン処理レイテンシへの影響 <1%、100 packets/sec を各宛先にフォワーディング可能
**Constraints**: 到達不能な宛先がメインパイプラインをブロックしないこと。メモリ使用量は宛先あたりバッファ32パケット以内
**Scale/Scope**: 宛先数: 1-10、チャンネル数: 1-10、パケットレート: ~100/sec (Raspberry Shake標準)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Check

| Principle | Status | Notes |
| --------- | ------ | ----- |
| I. 安定性と信頼性 | PASS | 到達不能な宛先でもパイプラインに影響なし。非ブロッキング設計。エラー処理徹底 |
| II. 厳密なテスト | PASS | ユニットテスト + E2E統合テスト（ローカルUDPリスナー）計画済み |
| III. 高いパフォーマンス | PASS | async `UdpSocket::send_to()`、バウンデッドチャンネル、独立タスクで低オーバーヘッド |
| IV. コードの明瞭性 | PASS | 既存の消費者パターン（Hue, Audio, SNS）に準拠した設計 |
| V. 日本語仕様 | PASS | 仕様・計画書を日本語で記載 |
| VI. 標準技術スタック | N/A | WebUI変更なし |
| VII. 自己検証の義務 | PASS | テスト実行・動作確認後にコミット |
| VIII. ブランチ運用 | PASS | `034-udp-forward`フィーチャーブランチで作業中 |

### Post-Design Check

| Principle | Status | Notes |
| --------- | ------ | ----- |
| I. 安定性と信頼性 | PASS | `ForwardError`型でエラーを明示。キュー溢れはドロップ+カウンタで対応 |
| II. 厳密なテスト | PASS | contracts/forward-module.md に具体的なテスト計画記載。チャンネルフィルタ・E2Eテストをカバー |
| III. 高いパフォーマンス | PASS | `try_send()`による非ブロッキング送信。パイプラインへの影響は`clone()`コストのみ |
| IV. コードの明瞭性 | PASS | `ForwardManager` 1ファイルに集約。既存パターンに沿った慣用的Rust |

## Project Structure

### Documentation (this feature)

```text
specs/034-udp-forward/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: design decisions
├── data-model.md        # Phase 1: entity definitions
├── quickstart.md        # Phase 1: usage guide
├── contracts/           # Phase 1: module contracts
│   └── forward-module.md
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── forward.rs          # NEW: ForwardManager, ForwardMsg, ForwardStats, forwarding tasks
│   ├── pipeline.rs         # MODIFIED: integrate forward_data() and forward_alarm() calls
│   ├── main.rs             # MODIFIED: initialize ForwardManager, pass to pipeline
│   └── lib.rs              # MODIFIED: add `pub mod forward;`
└── tests/
    └── test_forward.rs     # NEW: unit + integration tests
```

**Structure Decision**: 単一プロジェクト構成。新規ファイルは `src/forward.rs`（モジュール本体）と `tests/test_forward.rs`（テスト）の2ファイルのみ。既存ファイルへの変更は `pipeline.rs`, `main.rs`, `lib.rs` の3ファイルに限定。既存の消費者パターン（Hue, Audio, SNS）と同じ統合パターンを踏襲。

## Complexity Tracking

> 憲法違反なし。すべてのゲートをパス。

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (なし)    | —          | —                                   |
