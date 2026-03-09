# Research: アラート音声の順序再生（キューイング）

## R1: チャネル方式の選択

**Decision**: `std::sync::mpsc::channel` (標準ライブラリ)

**Rationale**:
- 再生スレッドは `sink.sleep_until_end()` でブロックするため、`std::thread` ベースが最適
- `std::sync::mpsc::Receiver::recv()` はブロッキング受信で、再生スレッドのイベントループに最適
- `Sender` は `Clone` 可能で、`Arc<AudioManager>` 経由の共有と互換
- 新規依存クレート不要

**Alternatives considered**:
- `tokio::sync::mpsc`: `blocking_recv()` で使用可能だが、tokio ランタイム外のスレッドで tokio チャネルを使う意味が薄い。オーバースペック。
- `crossbeam::channel`: 高性能だが新規依存追加が必要。本ユースケースでは標準 mpsc で十分。

## R2: AudioManager の設計変更

**Decision**: `AudioManager` 内に `Sender<String>` を保持し、`new()` で再生スレッドを起動

**Rationale**:
- 現在の `AudioManager` はステートレス（`_marker: ()` のみ）。`Sender` を追加するだけで最小変更
- 再生スレッドは `Receiver` を所有し、`recv()` → `play_file()` のループで順序再生を実現
- `play_file()` は既存ロジックをそのまま使用（再生ごとに fresh OutputStream）
- `AudioController = Arc<AudioManager>` の型エイリアスは変更不要

**Alternatives considered**:
- 外部キュー（`VecDeque` + `Mutex` + `Condvar`）: mpsc チャネルが同じ機能を内蔵しているため冗長
- rodio の `Sink::append()` による内蔵キュー: 永続 OutputStream が必要で ALSA POLLERR 問題が再発

## R3: pipeline.rs の呼び出し変更

**Decision**: `spawn_blocking` + `play_file()` を `queue_file()` 呼び出しに置換

**Rationale**:
- `queue_file()` は `Sender::send()` を内部で呼ぶだけで非ブロッキング
- `spawn_blocking` が不要になり、スレッド消費を削減
- 空パスのフィルタリングは `queue_file()` 内で行う（FR-004）

**Alternatives considered**:
- pipeline.rs 側でチャネル送信: AudioManager のカプセル化が崩れる。メソッドに封じ込める方が保守性が高い。

## R4: スレッド終了とリソース解放

**Decision**: `AudioManager` の `Drop` 時に `Sender` がドロップされ、再生スレッドの `recv()` が `Err` を返して自然終了

**Rationale**:
- 明示的な終了シグナル不要
- `std::sync::mpsc` の仕様: 全 `Sender` がドロップされると `recv()` は `Err(RecvError)` を返す
- プロセス終了時に確実にクリーンアップされる
