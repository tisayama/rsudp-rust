# Research: Streaming STA/LTA Trigger Calculation

**Feature**: 042-streaming-sta-lta | **Date**: 2026-03-02

## R1: Streaming vs Slice STA/LTA — Root Cause Analysis

**Decision**: フィルタ状態と STA/LTA 値をサンプル間で永続化するストリーミング方式を採用する

**Rationale**:
- `verify_stalta_compare.py` による検証で、フィルタ係数の違い（ハードコード vs scipy）は影響なし（A1=A2=72秒）
- スライス再計算方式のみが 159.5秒の遅延を再現（Test B=159.5秒）
- 原因はバンドパスフィルタのゼロ状態初期化によるバッファ先頭のトランジェント（過渡応答）が LTA を押し上げること
- ObsPy の `recursive_sta_lta` はデータストリーム全体に対して1パスで計算し、状態を永続化する方式

**Alternatives considered**:
1. バッファサイズを増やして過渡応答の影響を薄める → 根本解決にならず、メモリ増加
2. バッファ先頭のトランジェント部分を切り捨てる → 不正確、パラメータ調整が必要
3. フィルタのみ永続化し STA/LTA はバッファから計算 → 部分的改善だが不十分

## R2: ObsPy recursive_sta_lta アルゴリズム仕様

**Decision**: ObsPy の `recursive_sta_lta` と完全に同一のアルゴリズムを採用する

**Rationale**:
ObsPy の実装（C 拡張で高速化済み）:
```python
sta = 0.0
lta = 1e-99  # ゼロ除算防止
csta = 1.0 / nsta
clta = 1.0 / nlta

for i in range(ndat):
    sta = csta * a[i]**2 + (1 - csta) * sta
    lta = clta * a[i]**2 + (1 - clta) * lta
    ratio[i] = sta / lta
```

Rust での対応:
- `a[i]` はフィルタ済みサンプル（Biquad チェーン通過後）
- `a[i]**2` はエネルギー（二乗）
- STA/LTA は指数移動平均（EMA）
- `lta` の初期値 `1e-99` はゼロ除算防止（小さな正の値）

**Alternatives considered**:
- 移動平均ではなくスライディングウィンドウ平均 → ObsPy リファレンスと異なる
- `lta` 初期値を `0.0` にしてゼロ除算チェック → `1e-99` の方がシンプル

## R3: Biquad フィルタ状態の永続化方式

**Decision**: `Vec<Biquad>` を `StaLtaState` に含め、各 Biquad の `s1`, `s2` をサンプル間で保持する

**Rationale**:
- 現在の `Biquad::process()` は既に `&mut self` で状態を更新する設計（Direct Form II Transposed）
- 問題は `butter_bandpass_sos()` が毎回 `s1=0.0, s2=0.0` で新規作成すること
- 修正: 初回のみ作成し、`StaLtaState` に格納して永続化

**Implementation detail**:
- `StaLtaState` に `filters: Vec<Biquad>` フィールドを追加
- 初期化時に `butter_bandpass_sos()` で作成
- ギャップ検出時に `s1=0.0, s2=0.0` にリセット（係数 b0,b1,b2,a1,a2 はそのまま）

## R4: ギャップ検出と状態リセット

**Decision**: 1秒超のギャップ検出時に、フィルタ状態・STA・LTA・サンプルカウンタを全てリセットする

**Rationale**:
- 現在の実装: `state.buffer.clear()` でバッファをクリア
- ストリーミング方式: バッファ不要のため、代わりにフィルタ状態と STA/LTA 値をリセット
- リセット後は再度ウォームアップ期間（nlta サンプル = 30秒）が必要
- フィルタ係数（b0,b1,b2,a1,a2）はリセット不要（定数のため）

**Implementation detail**:
```rust
// ギャップ検出時のリセット
for bq in &mut state.filters {
    bq.s1 = 0.0;
    bq.s2 = 0.0;
}
state.sta = 0.0;
state.lta = 1e-99;
state.sample_count = 0;
```

## R5: 既存テストとの互換性

**Decision**: `integration_alert.rs` のテストを更新し、ストリーミング方式でも合格するようにする

**Rationale**:
- 既存テストは STA=1.0, LTA=10.0 の小さいパラメータで、1000 サンプルのウォームアップ後に 100 サンプルの高振幅を入力
- ストリーミング方式では LTA ウォームアップに `nlta` サンプル（1000）必要
- テストのウォームアップサンプル数が `nlta` と一致するため、ウォームアップ完了直後に高振幅が入る
- テストパラメータの微調整が必要になる可能性あり

**Risk mitigation**:
- テスト失敗時はウォームアップサンプル数を `nlta + 余裕` に増やす
- 高振幅区間の長さを必要に応じて調整
