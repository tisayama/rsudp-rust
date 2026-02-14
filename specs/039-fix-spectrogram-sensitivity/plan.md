# Implementation Plan: Fix Spectrogram Sensitivity

**Branch**: `039-fix-spectrogram-sensitivity` | **Date**: 2026-02-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/039-fix-spectrogram-sensitivity/spec.md`

## Summary

スペクトログラムのFFT出力をPSD正規化し、dB変換してから`^0.1`圧縮を適用する。現在のRust実装は生の`|X[k]|²`（線形）に対して`^0.1`を適用しているが、rsudp/matplotlibは`10*log10(PSD)`（dBスケール）に対して`^0.1`を適用している。この差異が非ピーク領域の過剰な明るさの原因。

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021)
**Primary Dependencies**: `rustfft` (FFT computation), `axum` (WebSocket), `tokio` (async)
**Storage**: In-memory (ring buffer, carry buffer for incremental FFT)
**Testing**: `cargo test` — 既存の filter テスト + 新規スペクトログラムテスト
**Target Platform**: Linux server (Docker)
**Project Type**: Web application (Rust backend + Next.js frontend)
**Performance Goals**: Real-time spectrogram computation for 4 channels at 100 sps
**Constraints**: No frontend changes required (u8 encoding is preserved)
**Scale/Scope**: 3関数の修正（compute_incremental_columns, compute_spectrogram, backfill normalization）

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
| ---- | ------ | ----- |
| I. 安定性と信頼性 | PASS | 理論的に正しいPSD計算への修正。短絡的解法ではなくmatplotlibの正確な再現 |
| II. 厳密なテスト | PASS | 既知入力に対するPSD/dB値の検証テストを追加 |
| III. 高いパフォーマンス | PASS | 追加計算は`log10`と除算のみ（FFT計算に対して無視できるコスト） |
| IV. コードの明瞭性 | PASS | PSD正規化ステップを明確にコメント |
| V. 日本語仕様 | PASS | 仕様書は日本語対応 |
| VI. 標準技術スタック | PASS | 既存のRust + Next.js構成を維持 |
| VII. 自己検証の義務 | PASS | ビルド・テスト・実行確認を実施 |
| VIII. ブランチ運用 | PASS | フィーチャーブランチで作業中 |

## Project Structure

### Documentation (this feature)

```text
specs/039-fix-spectrogram-sensitivity/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
rsudp-rust/src/
├── web/
│   ├── stream.rs        # compute_incremental_columns() + backfill normalization 修正
│   └── plot.rs          # compute_spectrogram() + compute_spectrogram_u8() 修正
└── (no other files changed)

rsudp-rust/tests/         # (or inline #[cfg(test)])
└── spectrogram_psd_test  # PSD/dB計算の検証テスト
```

**Structure Decision**: 既存の2ファイル（stream.rs, plot.rs）の修正のみ。新規ファイル不要。
