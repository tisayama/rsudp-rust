# Research: Fix Spectrogram Sensitivity

## Decision 1: PSD正規化方式

**Decision**: matplotlib互換のPSD正規化を実装する

**Rationale**: matplotlib `specgram()` のデフォルト動作（`mode='psd'`, `scale='dB'`）を正確に再現する。PSD正規化により、FFT出力が窓関数エネルギーとサンプリングレートで正規化され、物理的に意味のあるパワースペクトル密度値となる。

**Formula**:
```
PSD[k] = |X[k]|² / (Fs × Σ(window²))
One-sided: PSD[k] *= 2  (for 0 < k < NFFT/2)
```

**Alternatives considered**:
- 経験的なべき乗調整（`powf(0.05)` に変更）: 簡単だが理論的根拠がなく、サンプルレートやNFFT変更時に再調整が必要
- dBスケーリングのみ（PSD正規化なし）: 一部改善するが完全な再現にはならない

## Decision 2: dBスケーリングの適用タイミング

**Decision**: PSD正規化後、べき乗圧縮前にdB変換を適用する

**Rationale**: rsudpのコードフロー:
1. `specgram()` が `10 * log10(PSD)` を返す（dBスケール）
2. `sg ** (1/10)` でdB値をべき乗圧縮
3. `imshow()` で表示

Rustでも同じ順序を踏む必要がある。

**Processing pipeline**:
```
|X[k]|² → PSD正規化 → 10*log10(PSD) → dB値に^0.1 → 正規化 → u8変換
```

## Decision 3: 正規化方式（u8マッピング）

**Decision**: running_max方式を維持するが、dB値に対して適用する

**Rationale**:
- バックフィル: グローバル最大dB圧縮値で正規化（現行と同じ方針）
- ライブ: running_maxをdB圧縮値に対して追跡（decay=0.997/column維持）
- dB変換によりダイナミックレンジが大幅に圧縮されるため、running_maxの安定性が向上する

## Decision 4: log10のゼロ除算対策

**Decision**: PSD値にフロアを設定 `psd.max(1e-20)` してからlog10を適用

**Rationale**: 無音時やDC成分でPSD=0になる可能性がある。`1e-20` は `-200 dB` に相当し、実用上の最小値として十分。matplotlib内部でも同様のクランプが行われている。

## Decision 5: One-sided spectrum の扱い

**Decision**: DC成分(k=0)とNyquist成分(k=NFFT/2)以外を2倍にする

**Rationale**: 実数信号のFFTは対称性があり、片側スペクトルのみ使用する場合、エネルギー保存のために非DC/非Nyquist成分を2倍にする必要がある。matplotlibの`_spectral_helper()`が内部で行っている処理と同じ。

## 修正対象ファイルと関数

### 1. `rsudp-rust/src/web/stream.rs` — `compute_incremental_columns()`

現在:
```rust
let mags: Vec<f64> = buffer.iter().take(freq_bins)
    .map(|c| c.re * c.re + c.im * c.im)
    .collect();
// ...
let normalized = (mag_sq / norm_max).powf(0.1);
```

修正後:
```rust
// PSD正規化
let window_power_sum: f64 = hann.iter().map(|w| w * w).sum();
let psd_norm = sample_rate * window_power_sum;

let psd_db: Vec<f64> = buffer.iter().take(freq_bins).enumerate()
    .map(|(k, c)| {
        let mag_sq = c.re * c.re + c.im * c.im;
        let mut psd = mag_sq / psd_norm;
        // One-sided spectrum: double non-DC, non-Nyquist
        if k > 0 && k < freq_bins - 1 { psd *= 2.0; }
        // dB conversion
        10.0 * psd.max(1e-20).log10()
    })
    .collect();

// ^0.1 compression on dB values, then normalize
let compressed: Vec<f64> = psd_db.iter()
    .map(|&db| db.abs().powf(0.1) * db.signum())
    .collect();
```

### 2. `rsudp-rust/src/web/plot.rs` — `compute_spectrogram()`

同様にPSD正規化 + dB変換を追加。`compute_spectrogram_u8()`のべき乗圧縮もdB値に適用。

### 3. `rsudp-rust/src/web/stream.rs` — backfill normalization

バックフィルの手動正規化コードも同じPSD + dBパイプラインに統一。
