# Contract: Spectrogram Processing Pipeline

## Internal API (Rust functions)

### compute_incremental_columns()

**Signature** (updated):
```
fn compute_incremental_columns(
    state: &mut FftChannelState,
    new_samples: &[f64],
    hann: &[f64],
    fft: &Arc<dyn Fft<f64>>,
    sample_rate: f64,       // NEW: needed for PSD normalization
) -> Vec<Vec<u8>>
```

**Preconditions**:
- `hann.len() == NFFT`
- `sample_rate > 0`
- `fft` is planned for `NFFT`

**Postconditions**:
- Each returned column has exactly `NFFT / 2 + 1` elements
- Each element is in range `[0, 255]`
- Quiet background produces values predominantly below 80
- Peaks produce values above 180

**Processing**:
1. Combine carry_buf + new_samples
2. For each NFFT window (hop = HOP):
   - Mean subtract, apply Hanning window
   - FFT
   - PSD normalize: `|X[k]|² / (Fs × Σ(window²))`
   - One-sided correction: `×2` for `0 < k < NFFT/2`
   - dB: `10 * log10(max(psd, 1e-20))`
   - Compress: `|dB|^0.1 * sign(dB)`
3. Update running_max with decay (0.997/column)
4. Normalize compressed values to u8 using running_max

---

### compute_spectrogram()

**Signature** (updated):
```
fn compute_spectrogram(
    samples: &[f64],
    sample_rate: f64,
    nfft: usize,
    noverlap: usize,
) -> Spectrogram
```

**Change**: `Spectrogram.data` now contains dB-compressed values instead of raw magnitude squared.

---

## Binary Protocol (unchanged)

スペクトログラムのバイナリパケットフォーマットは変更なし:
```
[0x03][channelIdLen:u8][channelId:utf8][timestamp:i64le][sampleRate:f32le][hopDuration:f32le][frequencyBins:u16le][columnsCount:u16le][data:u8[]]
```

フロントエンドの変更は不要。u8値のセマンティクスが「より正確なスペクトログラム表現」に改善されるのみ。
