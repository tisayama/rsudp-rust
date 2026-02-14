# Tasks: Fix Spectrogram Sensitivity to Match rsudp

**Input**: Design documents from `/specs/039-fix-spectrogram-sensitivity/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: テストタスクを含む（Constitution Gate II: 厳密なテスト）

**Organization**: タスクはユーザーストーリー別に整理。US1がMVP。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: 並列実行可能（異なるファイル、依存なし）
- **[Story]**: 対応するユーザーストーリー（US1, US2）

---

## Phase 1: Setup

**Purpose**: 変更対象のコード確認と準備

- [x] T001 既存の `compute_incremental_columns()` と `compute_spectrogram()` の動作を `cargo test` で確認 in `rsudp-rust/`

---

## Phase 2: Foundational (PSD正規化ヘルパー)

**Purpose**: US1・US2共通の基盤ロジック

- [ ] T002 `compute_incremental_columns()` に `sample_rate: f64` パラメータを追加し、全呼び出し元を更新 in `rsudp-rust/src/web/stream.rs`
- [ ] T003 Hanning窓のパワー和 `Σ(window²)` を事前計算するコードを追加 in `rsudp-rust/src/web/stream.rs`

**Checkpoint**: コンパイルが通り、既存テストがパスすること

---

## Phase 3: User Story 1 - Spectrogram visual parity with rsudp (Priority: P1) 🎯 MVP

**Goal**: ライブスペクトログラムのPSD正規化 + dBスケーリング + べき乗圧縮をrsudpと一致させる

**Independent Test**: WebUIのスペクトログラムが、quiet条件で背景が暗く（u8 < 80が70%以上）、ピークが明瞭に表示される

### Tests for User Story 1

- [ ] T004 [P] [US1] PSD正規化の単体テスト: 既知の正弦波入力に対してPSD値がmatplotlib相当の値を返すことを検証 in `rsudp-rust/src/web/stream.rs` (#[cfg(test)])
- [ ] T005 [P] [US1] dB変換の単体テスト: PSD値 → dB変換 → べき乗圧縮の結果がゼロ入力・正弦波入力で期待値と一致 in `rsudp-rust/src/web/stream.rs` (#[cfg(test)])

### Implementation for User Story 1

- [ ] T006 [US1] `compute_incremental_columns()` のFFT後処理をPSD正規化に変更: `mag_sq / (sample_rate * window_power_sum)` + one-sided correction (`×2` for `0 < k < NFFT/2`) in `rsudp-rust/src/web/stream.rs`
- [ ] T007 [US1] `compute_incremental_columns()` にdB変換を追加: `10.0 * psd.max(1e-20).log10()` in `rsudp-rust/src/web/stream.rs`
- [ ] T008 [US1] `compute_incremental_columns()` のべき乗圧縮をdB値に適用するよう変更: `|dB|^0.1 * sign(dB)` で圧縮し、running_maxもdB圧縮値で追跡 in `rsudp-rust/src/web/stream.rs`
- [ ] T009 [US1] `compute_spectrogram()` にPSD正規化 + dB変換を追加 in `rsudp-rust/src/web/plot.rs`
- [ ] T010 [US1] `compute_spectrogram_u8()` のべき乗圧縮をdB値に適用するよう変更 in `rsudp-rust/src/web/plot.rs`
- [ ] T011 [US1] ビルド確認 (`cargo build --release`) + テスト実行 (`cargo test`) in `rsudp-rust/`
- [ ] T012 [US1] Docker Compose でリビルド・起動し、WebUIでスペクトログラムの背景暗さを目視確認

**Checkpoint**: ライブスペクトログラムが rsudp に近い見え方になること

---

## Phase 4: User Story 2 - Consistent backfill-to-live transition (Priority: P2)

**Goal**: バックフィルとライブデータの間でスペクトログラムの明るさ・コントラストに不連続がないこと

**Independent Test**: WebUIを開いてバックフィル→ライブの遷移点に目視で段差がないことを確認

### Implementation for User Story 2

- [ ] T013 [US2] バックフィルスペクトログラムの正規化コード（`handle_socket` 内の `compute_spectrogram` 呼び出し後）をPSD + dBパイプラインに統一 in `rsudp-rust/src/web/stream.rs`
- [ ] T014 [US2] バックフィルの `max_mag_sq` をdB圧縮値の最大値に変更し、ライブの `running_max` 初期値として引き継ぐ in `rsudp-rust/src/web/stream.rs`
- [ ] T015 [US2] Docker Compose で起動し、バックフィル→ライブの遷移点を目視確認

**Checkpoint**: バックフィルとライブの視覚的連続性が確保されていること

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: 最終品質確認とクリーンアップ

- [ ] T016 [P] 未使用の変数・import があればクリーンアップ in `rsudp-rust/src/web/stream.rs`, `rsudp-rust/src/web/plot.rs`
- [ ] T017 `cargo clippy` で警告なしを確認 in `rsudp-rust/`
- [ ] T018 全テスト実行 (`cargo test`) + Docker Compose でE2E確認

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: 依存なし
- **Phase 2 (Foundational)**: Phase 1 完了後
- **Phase 3 (US1)**: Phase 2 完了後 — MVPスコープ
- **Phase 4 (US2)**: Phase 3 完了後（US1のPSD/dBパイプラインに依存）
- **Phase 5 (Polish)**: Phase 4 完了後

### Within Each User Story

- テスト → 実装 → ビルド確認 → 目視確認
- PSD正規化 → dB変換 → べき乗圧縮（順序依存）

### Parallel Opportunities

- T004, T005: テスト作成は並列可能
- T009, T010: plot.rs の変更は stream.rs と並列可能（Phase 3内）
- T016: クリーンアップは他のPolishタスクと並列可能

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: Setup（既存テスト確認）
2. Phase 2: Foundational（sample_rate パラメータ追加）
3. Phase 3: US1（PSD + dB + 圧縮の修正）
4. **STOP and VALIDATE**: スペクトログラム背景が暗くなったことを確認
5. 問題なければ Phase 4 へ

### Incremental Delivery

1. Phase 1 + 2 → 基盤完了
2. Phase 3 (US1) → ライブスペクトログラム修正 → 確認
3. Phase 4 (US2) → バックフィル統一 → 確認
4. Phase 5 → クリーンアップ → コミット
