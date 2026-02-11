# Data Model: WebUI Spectrogram & rsudp-Compatible Plot

**Phase 1 Output** | **Date**: 2026-02-10

## Frontend Entities

### SpectrogramRenderer (TypeScript)

サーバーから受信したu8正規化済みスペクトログラムデータをCanvas描画する軽量レンダラー。FFT計算はサーバーサイド（rustfft）で実行済み。

| Field | Type | Description |
|-------|------|-------------|
| frequencyBins | number | 周波数ビン数（サーバーから通知、NFFT/2+1） |
| sampleRate | number | チャンネルのサンプルレート |
| imageData | ImageData | Canvas描画用のピクセルバッファ |
| columnCount | number | 現在のスペクトログラム列数 |
| maxColumns | number | Canvas幅に基づく最大列数 |

**State transitions**:
- `Empty` → `Rendering`: 最初のスペクトログラム列データ受信
- `Rendering` → `Rendering`: 新しい列追加（u8[] → Inferno LUT → RGBA → putImageData）、drawImage(self)でスクロール

**Validation rules**:
- 受信した列のu8配列長 == frequencyBins
- columnCount ≤ maxColumns（canvas width）

### ChannelPairState (TypeScript)

1チャンネルの波形＋スペクトログラムペアの描画状態。

| Field | Type | Description |
|-------|------|-------------|
| channelId | string | チャンネル識別子（例: "EHZ"） |
| waveformCanvas | HTMLCanvasElement | 波形描画用Canvas |
| spectrogramCanvas | HTMLCanvasElement | スペクトログラム描画用Canvas |
| spectrogramRenderer | SpectrogramRenderer | スペクトログラム描画状態（FFT計算はサーバーサイド） |
| ringBuffer | RingBuffer | サンプルデータリングバッファ |
| unitLabel | string | Y軸単位ラベル（"Counts", "Velocity (m/s)"等） |

### IntensityIndicatorState (TypeScript)

震度階級バッジの表示状態管理。

| Field | Type | Description |
|-------|------|-------------|
| visible | boolean | バッジ表示中かどうか |
| maxIntensity | number | 現在のアラート期間中の最大計測震度 |
| maxClass | string | 最大震度階級（"0", "1", ..., "5-", "5+", "6-", "6+", "7"） |
| triggerTime | Date \| null | アラート開始時刻 |
| resetTime | Date \| null | アラートリセット時刻 |
| fadeoutTimer | NodeJS.Timeout \| null | RESET後30秒タイマー |

**State transitions**:
- `Hidden` → `Active`: AlertStart受信
- `Active` → `Active`: Intensity更新（maxが上がった場合のみ更新）
- `Active` → `PostReset`: AlertEnd受信 → 30秒タイマー開始
- `PostReset` → `Hidden`: 30秒経過
- `PostReset` → `Active`: 新しいAlertStart受信（リセット＆再追跡）

### EventMarker (TypeScript — 既存拡張)

| Field | Type | Description |
|-------|------|-------------|
| type | 'Alarm' \| 'Reset' | マーカー種別 |
| timestamp | string | ISO 8601タイムスタンプ |
| channel | string | 対象チャンネル |

**Change**: 既存の`VisualAlertMarker`型を流用。色をrsudp互換に変更：Alarm=#4C8BF5(blue), Reset=#D72638(red)。

### PlotSettings (TypeScript — 既存拡張)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| active_channels | string[] | ['SHZ','EHZ'] | アクティブチャンネル（既存） |
| window_seconds | number | 90 | 表示ウィンドウ幅（既存、min=5, max=300） |
| auto_scale | boolean | true | Y軸自動スケール（既存） |
| show_spectrogram | boolean | true | スペクトログラム表示トグル（新規） |
| spectrogram_freq_min | number | 0 | スペクトログラム下限周波数Hz（新規） |
| spectrogram_freq_max | number | 50 | スペクトログラム上限周波数Hz（新規） |
| spectrogram_log_y | boolean | false | 対数Y軸（新規） |

## Backend Entities

### ChannelBuffer (Rust — 既存修正)

| Field | Type | Description |
|-------|------|-------------|
| data | VecDeque\<f64\> | サンプルデータ（既存） |
| end_time | DateTime\<Utc\> | バッファ末尾のタイムスタンプ（既存） |
| sample_rate | f64 | サンプルレート（既存） |

**Change**: `push_segment`の`max_len`パラメータを表示ウィンドウ設定に連動させる（PlotSettings.window_seconds * sample_rate）。

### SpectrogramComputer (Rust — 既存plot.rs拡張)

既存の`compute_spectrogram()`関数を活用し、WebSocket配信用にu8正規化機能を追加。

| Function | Signature | Description |
|----------|-----------|-------------|
| compute_spectrogram | `(samples: &[f64], sample_rate: f64, nfft: usize, noverlap: usize) -> Spectrogram` | 既存。PSD行列を返す |
| compute_spectrogram_u8 | `(samples: &[f64], sample_rate: f64, nfft: usize, noverlap: usize) -> SpectrogramU8` | 新規。power scaling (^1/10) + auto-normalize (0-255) 済みu8行列を返す |

**SpectrogramU8**:
```rust
pub struct SpectrogramU8 {
    pub frequency_bins: usize,   // NFFT/2 + 1
    pub sample_rate: f64,
    pub columns: Vec<Vec<u8>>,   // [time_column][frequency_bin] — u8 (0-255)
    pub timestamps: Vec<f64>,    // 各列の相対時刻（秒）
}
```

**Normalization process**:
1. `compute_spectrogram()`でPSD行列を取得
2. 全フレームのmax_mag_sqを求める（auto-scale）
3. 各値に `(mag_sq / max_mag_sq).powf(0.1) * 255.0` を適用して u8 にキャスト

### WebSocket Messages (Rust/TypeScript — 新規追加)

**BackfillRequest** (Client → Server):
```json
{ "type": "BackfillRequest", "last_timestamp": "2026-02-10T12:00:00Z" }
```
初回接続時は`last_timestamp`をnullにして全バッファデータを要求。

**BackfillResponse** (Server → Client):
バイナリWaveformパケット + バイナリSpectrogramパケットの両方を送信。最後に完了マーカー：
```json
{ "type": "BackfillComplete", "channels": ["EHZ", "ENE", "ENN", "ENZ"] }
```

**Binary Spectrogram Packet** (Server → Client — 新規):
```
[0x03] [channelIdLen:u8] [channelId:utf8] [timestamp:i64le(μs)] [sampleRate:f32le] [frequencyBins:u16le] [columnsCount:u16le] [data:u8[columnsCount × frequencyBins]]
```
- type byte `0x03` でWaveform(`0x00`)と区別
- dataは列優先（column-major）: 各列がfrequencyBins個のu8値（0=最小パワー、255=最大パワー）
- power scaling (`^1/10`) + auto-normalize済み

## Inferno Colormap LUT

256エントリのRGB配列。インデックス0=最小パワー（ほぼ黒）、インデックス255=最大パワー（明るい黄色）。
matplotlibの`plt.get_cmap('inferno')`から生成し、TypeScript定数として埋め込む。

## JMA Intensity Color Scale

| Class | Threshold | Color (hex) | RGB |
|-------|-----------|-------------|-----|
| 1 | intensity < 1.5 | #F2F2FF | (242, 242, 255) |
| 2 | intensity < 2.5 | #00AAFF | (0, 170, 255) |
| 3 | intensity < 3.5 | #0041FF | (0, 65, 255) |
| 4 | intensity < 4.5 | #FAE696 | (250, 230, 150) |
| 5- | intensity < 5.0 | #FFE600 | (255, 230, 0) |
| 5+ | intensity < 5.5 | #FF9900 | (255, 153, 0) |
| 6- | intensity < 6.0 | #FF2800 | (255, 40, 0) |
| 6+ | intensity < 6.5 | #A50021 | (165, 0, 33) |
| 7 | intensity >= 6.5 | #B40068 | (180, 0, 104) |

## Channel Sort Order

チャンネル名の末尾文字で以下の優先順位でソート：
1. Z-ending（例: EHZ, ENZ）
2. E-ending（例: EHE, ENE）
3. N-ending（例: EHN, ENN）
4. その他（アルファベット順）

```typescript
function channelSortKey(ch: string): [number, string] {
  if (ch.endsWith('Z')) return [0, ch];
  if (ch.endsWith('E')) return [1, ch];
  if (ch.endsWith('N')) return [2, ch];
  return [3, ch];
}
```
