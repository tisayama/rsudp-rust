# Implementation Plan: Classic STA/LTA Algorithm

**Branch**: `043-classic-stalta` | **Date**: 2026-03-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/043-classic-stalta/spec.md`

## Summary

STA/LTAアルゴリズムをRecursive (EMA) 方式からClassic (スライディングウィンドウ平均) 方式に変更する。EMA方式の「無限記憶」特性により通常ノイズでratio 0.4〜1.5で振動し誤報が多発する問題を、有限記憶のスライディングウィンドウ平均で解決する。リングバッファ + 累積和によるO(1)更新を実装し、定期的な累積和再計算で長期運用時の数値安定性を担保する。

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021)
**Primary Dependencies**: `chrono` (時刻処理), `serde` (シリアライゼーション), `tracing` (ログ) — すべて既存依存
**Storage**: インメモリ (リングバッファ: `VecDeque<f64>`)
**Testing**: `cargo test` + MiniSEEDリファレンスデータ (shindo0.mseed, normal.mseed, normal2.mseed)
**Target Platform**: Linux (aarch64/x86_64)
**Project Type**: Single (Rustバイナリ)
**Performance Goals**: O(1) per sample (現行EMA方式と同等)
**Constraints**: メモリ増加 24KB以内 (3000 × 8バイト), 既存API signature不変
**Scale/Scope**: 単一ファイル (`trigger.rs`) のリファクタリング + テスト更新

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| 原則 | 適合性 | 備考 |
|------|--------|------|
| I. 安定性と信頼性 | PASS | Classic STA/LTAは理論的に正しいアルゴリズム。有限記憶で誤報を防止。 |
| II. 厳密なテスト | PASS | shindo0.mseed (地震検出), normal.mseed/normal2.mseed (誤報ゼロ) で検証。 |
| III. 高いパフォーマンス | PASS | O(1)更新を維持。メモリ増加は24KBのみ。 |
| IV. コードの明瞭性 | PASS | EMAの暗黙的な無限記憶よりスライディングウィンドウの方が挙動が直感的。 |
| V. 日本語仕様 | PASS | 仕様書は日本語で作成済み。 |
| VI. 標準技術スタック | N/A | WebUI変更なし。 |
| VII. 自己検証の義務 | PASS | cargo test + MiniSEEDデータによる自動検証。 |
| VIII. ブランチ運用 | PASS | `043-classic-stalta` ブランチで作業中。 |

**結果: すべてPASS — Phase 0に進行可能。**

## Project Structure

### Documentation (this feature)

```text
specs/043-classic-stalta/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   └── trigger.rs           # PRIMARY: StaLtaState構造体 + add_sample() — Classic STA/LTA実装
├── tests/
│   ├── test_stalta_mseed.rs  # UPDATE: shindo0.mseed検証テスト
│   ├── test_normal_mseed.rs  # UPDATE: normal.mseed誤報ゼロテスト
│   └── integration_alert.rs  # UPDATE: 合成データ統合テスト
└── Cargo.toml               # NO CHANGE: 新規依存なし
```

**Structure Decision**: 単一ファイル (`trigger.rs`) のアルゴリズム変更。テストファイルは既存の更新のみ。
