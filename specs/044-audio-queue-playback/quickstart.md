# Quickstart: アラート音声の順序再生（キューイング）

## 前提条件

- Rust 1.7x (stable)
- ALSA 対応の音声出力デバイス（Raspberry Pi または Linux デスクトップ）
- テスト用 MP3/WAV ファイル

## ビルド・テスト

```bash
# ビルド
cargo build --manifest-path rsudp-rust/Cargo.toml

# テスト実行
cargo test --manifest-path rsudp-rust/Cargo.toml

# Clippy
cargo clippy --manifest-path rsudp-rust/Cargo.toml
```

## 検証シナリオ

### シナリオ 1: 順序再生の確認

1. ALARM 音声ファイル（3秒以上）と RESET 音声ファイルを用意
2. `rsudp.toml` で `alertsound.enabled = true`、`trigger_file` と `default_reset_file` を設定
3. shindo0.mseed をストリーマーで再生:
   ```bash
   cargo run --manifest-path rsudp-rust/Cargo.toml --bin streamer -- \
     --file references/mseed/shindo0.mseed --addr 127.0.0.1:8888 --speed 10
   ```
4. ALARM 音 → RESET 音が順番に再生され、重ならないことを確認

### シナリオ 2: キュー空状態での即座再生

1. システムをアイドル状態で起動
2. ALARM イベントを発火させる
3. 音声が即座に（体感遅延なく）再生開始されることを確認

### シナリオ 3: 連続イベント

1. fdsnws.mseed（2回の地震イベント）をストリーマーで高速再生:
   ```bash
   cargo run --manifest-path rsudp-rust/Cargo.toml --bin streamer -- \
     --file references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --speed 20
   ```
2. 複数の ALARM/RESET 音がキュー順に再生されることを確認

### シナリオ 4: 長時間稼働

1. システムを起動し、1時間以上アイドル状態を維持
2. その後 ALARM イベントを発火
3. ALSA エラーなく正常に再生されることを確認（ログに POLLERR なし）
