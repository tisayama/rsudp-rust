# Research: Classic STA/LTA Algorithm

## R1: Classic vs Recursive STA/LTA — アルゴリズム比較

### Decision
Classic (スライディングウィンドウ平均) STA/LTAを採用する。

### Rationale
- **Classic STA/LTA**: `STA = (1/nsta) * Σ energy[i-nsta+1:i+1]`, `LTA = (1/nlta) * Σ energy[i-nlta+1:i+1]`
  - 有限記憶: LTAは直近nlta個のサンプルのみを反映。30秒以前の情報は完全に忘却。
  - 定常ノイズでratio ≈ 1.0に安定。分散が小さい。
  - threshold=1.1でも通常ノイズでの誤報がほぼ発生しない。

- **Recursive (EMA) STA/LTA**: `sta = csta * energy + (1-csta) * sta`
  - 無限記憶: 指数減衰するが過去のすべてのサンプルが影響。
  - 静穏期後の通常レベル復帰でLTAが追従遅延 → ratioが1.5以上に振動。
  - threshold=1.1で60分あたり17-21回の誤報が発生。

### Alternatives Considered
1. **EMA + 閾値引き上げ**: threshold=3.95 (Python rsudpデフォルト) にすれば誤報は減るが、ユーザーの既存設定(1.1)との互換性がない。
2. **EMA + duration debounce**: duration > 0で一時的なスパイクをフィルタリングできるが、数百秒持続する誤報パターンには無効。
3. **Python rsudp互換のウィンドウ再計算方式**: パケットごとにフィルタ+STA/LTAをゼロから再計算する方式。Python互換だがO(3000)/パケットで計算効率が悪く、042で除去した旧スライスモードへの回帰になる。

## R2: リングバッファの実装方式

### Decision
`VecDeque<f64>` を使用し、固定容量nlta個のリングバッファとして運用する。

### Rationale
- Rustの標準ライブラリに含まれ、外部依存なし。
- `push_back()` + `pop_front()` でO(1)のFIFO操作。
- `with_capacity(nlta)` で事前確保により再確保なし。
- `iter().sum()` で定期的な累積和再計算が容易。

### Alternatives Considered
1. **固定長配列 + インデックス**: `[f64; 3000]` + write pointer。ヒープ確保なしだがサイズが設定で変わる場合に不便。定数でないサイズの配列はRustでは不可。
2. **`Vec<f64>` + 手動インデックス**: 可能だが `VecDeque` がFIFO操作のセマンティクスを直接提供するため不要。

## R3: 累積和の数値安定性

### Decision
累積和の加算・減算方式をメイン計算とし、10,000サンプル (100秒) ごとにリングバッファ全体から再計算する。

### Rationale
- f64の有効桁数は約15-16桁。energy値のオーダーが1e-10〜1e+10の範囲であっても、10,000回の加減算では累積誤差は無視できるレベル。
- 100秒ごとの再計算コストはO(3000) ≈ 30μs程度で、100秒に1回の頻度では性能影響なし。
- 24時間 = 86,400秒 = 864回の再計算。各再計算で誤差がリセットされるため長期運用でも安定。

### Alternatives Considered
1. **Kahan加算アルゴリズム**: 累積誤差を補償する高精度加算。実装が複雑で、定期再計算の方がシンプル。
2. **再計算なし**: f64の精度で実用上問題ない可能性が高いが、24時間以上の連続運用で保証できないためリスク。

## R4: STA/LTAの計算方式 — 部分和か累積和か

### Decision
LTA用の累積和 `sum_lta` のみを保持し、STAはリングバッファの末尾nsta個から計算する。

### Rationale
- LTAリングバッファにはnlta個のenergyが格納されている。
- STAは「直近nsta個の平均」= リングバッファの末尾nsta個の平均。
- `sum_sta` を別途持つ場合、nsta個前の値を参照するためにリングバッファ上のインデックス計算が必要で複雑。
- 代わりに `sum_lta` (全nlta個の合計) と `sum_lta_minus_sta` (先頭nlta-nsta個の合計) を持つことで `sum_sta = sum_lta - sum_lta_minus_sta` と計算できるが、2つの累積和の管理が必要。
- **最もシンプルな方式**: `sum_lta` のみを累積和で管理し、`sum_sta` はリングバッファの末尾nsta個を直接合計。nsta=600なので合計のコストはO(600)だが、100SPSで毎秒100回 × 600回 = 60,000回の加算は現代CPUでは無視できるレベル (< 100μs/秒)。

### Alternatives Considered
1. **sum_sta + sum_lta の二重累積和**: O(1)更新だが管理が複雑。2つの累積和の同期と再計算が必要。
2. **sum_lta のみ + STAインデックス参照**: リングバッファのnsta個前の要素を参照して差分更新。`VecDeque` のインデックスアクセスはO(1)だが、ウォームアップ中のnsta個未満の場合の分岐が複雑。

**最終方針**: `sum_lta` を累積和で管理 (O(1)更新)。`sum_sta` はリングバッファ末尾nsta個の直接合計 (O(nsta)更新)。定期再計算で `sum_lta` を補正。
