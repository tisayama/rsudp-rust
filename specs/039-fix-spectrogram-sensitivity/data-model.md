# Data Model: Fix Spectrogram Sensitivity

## Entities

### SpectrogramColumn (modified)

各FFT窓から生成されるスペクトログラムの1列分のデータ。

| Field | Type | Description |
| ----- | ---- | ----------- |
| psd_db | Vec<f64> | PSD正規化 + dB変換済みのパワー値（frequency_bins個） |
| compressed | Vec<f64> | dB値にべき乗圧縮(`^0.1`)を適用した値 |
| u8_column | Vec<u8> | running_maxで正規化し0-255にマッピングした最終出力 |

### Processing Pipeline (updated)

```
Input: raw FFT complex output X[k]

Step 1: Magnitude squared
  mag_sq = X[k].re² + X[k].im²

Step 2: PSD normalization
  psd = mag_sq / (Fs × Σ(window²))
  if 0 < k < NFFT/2: psd *= 2  (one-sided correction)

Step 3: dB conversion
  psd_db = 10 × log₁₀(max(psd, 1e-20))

Step 4: Power-law compression
  compressed = |psd_db|^0.1 × sign(psd_db)

Step 5: Normalize to u8
  u8_val = ((compressed - min) / (max - min) × 255).clamp(0, 255)
```

### FftChannelState (unchanged structure, changed semantics)

| Field | Type | Description |
| ----- | ---- | ----------- |
| carry_buf | Vec<f64> | 次のバッチに引き継ぐ未処理サンプル |
| running_max | f64 | dB圧縮値のrunning maximum（以前は線形mag_sqの最大値） |

## State Transitions

なし（スペクトログラムはステートレスなFFT計算。carry_bufとrunning_maxは継続状態だが状態遷移ではない）。

## Validation Rules

- PSD値は0以上（パワーは負にならない）
- dB値は負になりうる（弱い信号）
- dB圧縮後の値も負になりうるため、正規化はmin-max方式で行う
- u8出力は常に0-255の範囲
