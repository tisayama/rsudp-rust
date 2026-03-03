# Quickstart: Classic STA/LTA Algorithm 検証手順

## 前提条件

- Rust toolchain (cargo, rustc)
- リファレンスデータ: `references/mseed/shindo0.mseed`, `references/mseed/normal.mseed`, `references/mseed/normal2.mseed`

## 検証シナリオ

### 1. ビルド確認

```bash
cargo build --manifest-path rsudp-rust/Cargo.toml
```

期待結果: コンパイル成功、warning最小。

### 2. 地震検出テスト (US2)

```bash
cargo test --manifest-path rsudp-rust/Cargo.toml test_streaming_stalta_shindo0 -- --nocapture
```

期待結果:
- ALARM 1回 (09:00:44付近)
- RESET 1回 (09:01:56付近)
- ALARM→RESET = 72s ±5s
- max ratio > threshold (1.1を十分超過)

### 3. 誤報ゼロテスト (US1)

```bash
cargo test --manifest-path rsudp-rust/Cargo.toml test_normal -- --nocapture
```

期待結果:
- normal.mseed: ALARM = 0回
- normal2.mseed: ALARM = 0回

### 4. 統合テスト

```bash
cargo test --manifest-path rsudp-rust/Cargo.toml integration_alert -- --nocapture
```

期待結果: 合成データでのALARM→RESET サイクルが正常動作。

### 5. Clippy

```bash
cargo clippy --manifest-path rsudp-rust/Cargo.toml
```

期待結果: trigger.rs にwarning/errorなし。

### 6. E2E検証 (オプション)

```bash
cargo build --release --manifest-path rsudp-rust/Cargo.toml
# ターミナル1: rsudp-rust起動
./rsudp-rust/target/release/rsudp-rust
# ターミナル2: ストリーマーでnormal.mseedを送信
cargo run --manifest-path rsudp-rust/Cargo.toml --bin streamer -- --file references/mseed/normal.mseed --addr 127.0.0.1:8888 --speed 10
```

期待結果: 60分間のストリーミングでALARM発火なし。

## トラブルシューティング

- **テストがnormal.mseedでALARMを検出した場合**: Classic STA/LTAの実装を確認。sum_staの計算がリングバッファ末尾nsta個の合計になっているか確認。
- **shindo0.mseedでALARM→RESETが72s±5sから外れる場合**: ウォームアップ期間 (nlta) とratio計算のタイミングを確認。
- **数値ドリフトが疑われる場合**: 定期再計算の間隔 (10000サンプル) を短くして検証。
