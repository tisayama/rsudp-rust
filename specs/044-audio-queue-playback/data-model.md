# Data Model: アラート音声の順序再生（キューイング）

## Entities

### AudioManager (改修)

音声再生のキューイングを管理する。

| Field | Type | Description |
|-------|------|-------------|
| sender | Sender\<String\> | 再生キューへのファイルパス送信端 |

**変更点**: `_marker: ()` を `sender: Sender<String>` に置換

### 再生スレッド (新規、AudioManager 内部)

`AudioManager::new()` で起動される専用スレッド。

**ライフサイクル**:
1. `new()` でスレッド起動、`Receiver<String>` を所有
2. `recv()` でブロッキング待機
3. ファイルパス受信 → 既存 `play_file()` ロジックで再生（fresh OutputStream）
4. 再生完了 → 2 に戻る
5. 全 `Sender` ドロップ → `recv()` が `Err` → スレッド終了

### 再生キュー (暗黙)

`std::sync::mpsc::channel` の内部バッファがFIFOキューとして機能する。明示的なデータ構造は不要。

| Property | Value |
|----------|-------|
| 容量 | 無制限 (unbounded) |
| 順序保証 | FIFO |
| スレッド安全性 | 送信側: Clone 可能、受信側: 単一スレッド所有 |

## Relationships

```
Pipeline --queue_file()--> AudioManager.sender --mpsc--> 再生スレッド --play_file()--> ALSA
```

## State Transitions

```
AudioManager:
  Created (new) → Active (sender alive) → Dropped (sender dropped → thread exits)

再生スレッド:
  Waiting (recv blocked) → Playing (play_file executing) → Waiting (loop back)
                                                         → Terminated (RecvError)
```
