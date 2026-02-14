# Quickstart: Fix Spectrogram Sensitivity

## 概要

スペクトログラムのFFT→u8変換パイプラインに、PSD正規化とdBスケーリングを追加する。

## 修正ファイル

1. **`rsudp-rust/src/web/stream.rs`** — `compute_incremental_columns()` + backfill section
2. **`rsudp-rust/src/web/plot.rs`** — `compute_spectrogram()` + `compute_spectrogram_u8()`

## 実装手順

### Step 1: compute_incremental_columns() の修正

```rust
// 窓関数パワー和を事前計算（関数外またはキャッシュ）
let window_power_sum: f64 = hann.iter().map(|w| w * w).sum();
let psd_norm = sample_rate * window_power_sum;

// FFT後のマグニチュード計算を置き換え
let psd_db: Vec<f64> = buffer.iter().take(freq_bins).enumerate()
    .map(|(k, c)| {
        let mag_sq = c.re * c.re + c.im * c.im;
        let mut psd = mag_sq / psd_norm;
        if k > 0 && k < freq_bins - 1 { psd *= 2.0; }
        10.0 * psd.max(1e-20).log10()
    })
    .collect();
```

### Step 2: べき乗圧縮をdB値に適用

```rust
// 現在: (mag_sq / norm_max).powf(0.1)
// 修正: dB値にべき乗圧縮、その後正規化
let compressed: f64 = db_val.abs().powf(0.1) * db_val.signum();
let normalized = (compressed / running_max_compressed).clamp(0.0, 1.0);
let u8_val = (normalized * 255.0) as u8;
```

### Step 3: compute_spectrogram() の修正

plot.rs の `compute_spectrogram()` にも同じPSD正規化 + dB変換を適用。

### Step 4: テスト追加

既知の正弦波入力に対して、PSD値がmatplotlibの出力と一致することを検証するテストを追加。

## ビルド・テスト

```bash
cd rsudp-rust
cargo test
cargo build --release
```

## 検証

1. Docker Compose でバックエンドを起動
2. Streamer でデータ送信
3. WebUI でスペクトログラムを目視確認
4. rsudp と並べて比較
