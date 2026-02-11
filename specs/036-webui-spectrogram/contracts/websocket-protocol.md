# WebSocket Protocol Contract: Backfill Extension

**Phase 1 Output** | **Date**: 2026-02-10

## Overview

既存WebSocketプロトコル（バイナリWaveform + JSON Alert/Intensity）に、バックフィルリクエスト/レスポンスを追加する。

## Existing Messages (unchanged)

### Binary Waveform Packet (Server → Client) — Type 0x00
```
[0x00] [channelIdLen:u8] [channelId:utf8] [timestamp:i64le(μs)] [sampleRate:f32le] [samplesCount:u32le] [samples:f32le[]]
```

### JSON Alert Messages (Server → Client)
```json
{ "type": "AlertStart", "data": { "id": "uuid", "channel": "EHZ", "timestamp": "ISO8601" } }
{ "type": "AlertEnd", "data": { "id": "uuid", "channel": "EHZ", "timestamp": "ISO8601", "max_ratio": 5.2, "message": "..." } }
```

### JSON Intensity Message (Server → Client)
```json
{ "type": "Intensity", "data": { "timestamp": "ISO8601", "intensity": 3.7, "shindo_class": "4" } }
```

## New Binary Message

### Binary Spectrogram Packet (Server → Client) — Type 0x03

サーバーサイドFFT（rustfft）で計算し、power scaling (`^1/10`) + auto-normalize (0-255) 済みのu8スペクトログラムデータ。

```
[0x03] [channelIdLen:u8] [channelId:utf8] [timestamp:i64le(μs)] [sampleRate:f32le] [frequencyBins:u16le] [columnsCount:u16le] [data:u8[columnsCount × frequencyBins]]
```

| Field | Type | Size | Description |
|-------|------|------|-------------|
| type | u8 | 1 | 常に `0x03`（Spectrogramパケット識別子） |
| channelIdLen | u8 | 1 | チャンネルID文字列のバイト長 |
| channelId | utf8 | channelIdLen | チャンネル識別子（例: "EHZ"） |
| timestamp | i64le | 8 | パケット先頭のタイムスタンプ（μs精度、Unix epoch） |
| sampleRate | f32le | 4 | サンプルレート（Hz） |
| frequencyBins | u16le | 2 | 周波数ビン数（NFFT/2 + 1、例: 65 for NFFT=128） |
| columnsCount | u16le | 2 | 時間列数 |
| data | u8[] | columnsCount × frequencyBins | 正規化済みパワー値（0=最小、255=最大）列優先配置 |

**パケットサイズ例**（NFFT=128, overlap=90%, 100Hz, 1秒分）:
- frequencyBins = 65, columnsCount ≈ 8
- data = 8 × 65 = 520 bytes + header ≈ 540 bytes/秒/チャンネル

**ライブストリーミング**: サーバーはFFTフレーム生成ごとに即座にSpectrogramパケットを送信。バックフィル時はバッファ全体分を一括送信。

**データ配置（列優先）**:
```
data[0..frequencyBins]                          → column 0 (oldest time)
data[frequencyBins..2*frequencyBins]            → column 1
...
data[(columnsCount-1)*frequencyBins..end]       → column N-1 (newest time)
```
各列内はindex 0 = 0 Hz（DC成分）、index N = Nyquist周波数。

## New Messages

### BackfillRequest (Client → Server)

クライアントがWebSocket接続確立直後に送信する。

**初回接続時（データなし）:**
```json
{ "type": "BackfillRequest" }
```

**再接続時（前回データあり）:**
```json
{ "type": "BackfillRequest", "last_timestamp": "2026-02-10T12:00:00.000000Z" }
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type | string | Yes | 常に "BackfillRequest" |
| last_timestamp | string \| null | No | 最後に受信したサンプルのISO 8601タイムスタンプ（μs精度）。省略時はバッファ全体を要求 |

### BackfillResponse (Server → Client)

サーバーがBackfillRequestに応答する。

**フロー:**
1. サーバーは各チャンネルの`ChannelBuffer`から`last_timestamp`以降のデータを抽出
2. バイナリWaveformパケット（0x00）でサンプルデータを送信
3. 抽出サンプルに対して`compute_spectrogram_u8()`を実行し、バイナリSpectrogramパケット（0x03）で送信
4. 全チャンネル送信完了後、完了マーカーを送信

**完了マーカー:**
```json
{ "type": "BackfillComplete", "data": { "channels": ["EHZ", "ENE", "ENN", "ENZ"] } }
```

| Field | Type | Description |
|-------|------|-------------|
| type | string | 常に "BackfillComplete" |
| data.channels | string[] | バックフィルデータを送信したチャンネルID一覧 |

## Protocol Sequence

### Initial Connection
```
Client                          Server
  |--- WebSocket Connect -------->|
  |<-- Connection Established ----|
  |--- BackfillRequest {} ------->|  (no last_timestamp)
  |<-- Waveform(EHZ, t0, [...]) -|  (backfill waveform)
  |<-- Spectrogram(EHZ, t0, [u8])|  (backfill spectrogram)
  |<-- Waveform(ENE, t0, [...]) -|
  |<-- Spectrogram(ENE, t0, [u8])|
  |<-- Waveform(ENN, t0, [...]) -|
  |<-- Spectrogram(ENN, t0, [u8])|
  |<-- Waveform(ENZ, t0, [...]) -|
  |<-- Spectrogram(ENZ, t0, [u8])|
  |<-- BackfillComplete ----------|
  |<-- Waveform(EHZ, t1, [...]) -|  (live streaming continues)
  |<-- Spectrogram(EHZ, t1,[u8])-|  (live spectrogram continues)
  |<-- ...                        |
```

### Reconnection
```
Client                          Server
  |--- WebSocket Connect -------->|
  |<-- Connection Established ----|
  |--- BackfillRequest            |
  |    { last_timestamp: t_last } |
  |<-- Waveform(EHZ, t_last+1..)-|  (gap waveform)
  |<-- Spectrogram(EHZ, gap,[u8])|  (gap spectrogram)
  |<-- ...                        |
  |<-- BackfillComplete ----------|
  |<-- Waveform(EHZ, t_now, [..])|  (live streaming resumes)
  |<-- Spectrogram(EHZ, ..,[u8])-|  (live spectrogram resumes)
```

## Server Implementation Notes

- `handle_socket`関数を修正し、接続直後にクライアントからの最初のメッセージを待つ
- BackfillRequest受信後、`waveform_buffers`からデータを抽出して送信
- 波形データ抽出後、`compute_spectrogram_u8()`でスペクトログラムを計算しSpectrogramパケットとして送信
- バックフィル送信中もbroadcastチャンネルの購読は開始し、バックフィル完了後にライブデータの転送を開始
- `last_timestamp`がバッファの開始時刻より古い場合、バッファの全データを返す
- ライブストリーミング時: 新しいWaveformパケットに対して増分FFT計算を行い、対応するSpectrogramパケットを追従送信
- FFTパラメータ: NFFT=128, overlap=90%（≤60秒ウィンドウ）or 97.5%（>60秒ウィンドウ）

## Client Implementation Notes

- `useWebSocket.ts`を修正し、`onopen`時にBackfillRequestを送信
- バイナリメッセージの先頭バイトでパケット種別を判定: `0x00`=Waveform, `0x03`=Spectrogram
- Waveformパケットは既存ハンドラで処理（RingBufferに格納）
- Spectrogramパケットは`SpectrogramRenderer`に渡してu8→Inferno LUT→ImageData変換→Canvas描画
- `last_timestamp`はlocalStorageまたはref変数で保持
- BackfillComplete受信までは「Loading」状態を示すことも可能（オプション）
