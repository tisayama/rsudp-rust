# Data Model: Classic STA/LTA Algorithm

## Entity: StaLtaState (変更)

チャネルごとのSTA/LTA計算状態を保持する構造体。

### フィールド変更

| フィールド | 型 | 変更 | 説明 |
|-----------|-----|------|------|
| `triggered` | `bool` | 不変 | 現在アラート発火中かどうか |
| `max_ratio` | `f64` | 不変 | アラート期間中の最大ratio |
| `last_timestamp` | `Option<DateTime<Utc>>` | 不変 | 直前サンプルのタイムスタンプ (ギャップ検出用) |
| `exceed_start` | `Option<DateTime<Utc>>` | 不変 | threshold超過開始時刻 (duration用) |
| `is_exceeding` | `bool` | 不変 | 現在threshold超過中かどうか |
| `filters` | `Vec<Biquad>` | 不変 | バンドパスフィルタ状態 (永続化) |
| ~~`sta`~~ | ~~`f64`~~ | **削除** | ~~EMA方式のSTA値~~ |
| ~~`lta`~~ | ~~`f64`~~ | **削除** | ~~EMA方式のLTA値~~ |
| `energy_buffer` | `VecDeque<f64>` | **新規** | フィルタ済みenergy値のリングバッファ (容量: nlta) |
| `sum_lta` | `f64` | **新規** | LTAウィンドウ全体のenergy累積和 |
| `sample_count` | `usize` | 不変 | 累積サンプル数 (ウォームアップ判定 + 定期再計算タイミング) |

### 初期状態

```
StaLtaState {
    triggered: false,
    max_ratio: 0.0,
    last_timestamp: None,
    exceed_start: None,
    is_exceeding: false,
    filters: butter_bandpass_sos(4, highpass, lowpass, 100.0),
    energy_buffer: VecDeque::with_capacity(nlta),
    sum_lta: 0.0,
    sample_count: 0,
}
```

### ギャップリセット後の状態

```
filters: [各Biquadのs1=0.0, s2=0.0にリセット。係数は不変]
energy_buffer: clear()
sum_lta: 0.0
sample_count: 0
```

## 計算フロー (per sample)

```
1. val = sample
2. for section in filters: val = section.process(val)
3. energy = val * val
4. energy_buffer.push_back(energy)
5. sum_lta += energy
6. if energy_buffer.len() > nlta:
     removed = energy_buffer.pop_front()
     sum_lta -= removed
7. sample_count += 1
8. if sample_count % 10000 == 0:
     sum_lta = energy_buffer.iter().sum()    // 定期再計算
9. if sample_count < nlta:
     ratio = 0.0    // ウォームアップ中
   else:
     lta = sum_lta / nlta
     sta = energy_buffer[nlta-nsta..nlta].iter().sum() / nsta
     ratio = sta / lta
10. [既存トリガーロジック (不変)]
```

## 不変エンティティ

以下のエンティティは変更なし:

- **TriggerConfig**: 設定パラメータ (sta_sec, lta_sec, threshold, reset_threshold, highpass, lowpass, target_channel, duration)
- **TriggerManager**: チャネル→StaLtaState のHashMap管理
- **AlertEvent**: アラートイベント構造体
- **AlertEventType**: Trigger / Reset / Status
- **Biquad**: バンドパスフィルタセクション
- **butter_bandpass_sos()**: フィルタ係数生成関数
