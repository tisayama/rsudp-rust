# rsudp-rust

[English](#english) | [日本語](#japanese)

---

<a name="english"></a>
## English

Rust implementation of the `rsudp` seismic monitoring application. High performance, stability, and reliability for real-time seismic data processing.

### Key Features
- **UDP Data Receiver**: Efficiently receives MiniSEED or JSON data packets from Raspberry Shake or other sources.
- **Pure Rust MiniSEED Parser**: Custom parser for seismic data records.
- **STA/LTA Alerting**: Real-time event detection using Short-Term Average / Long-Term Average algorithms.
- **Seismic Intensity Calculation**: Calculates JMA (Japan Meteorological Agency) seismic intensity from 3-component acceleration data.
- **Web Interface**: Real-time waveform plotting and alert history management via a modern WebUI (Next.js + Axum).
- **Comprehensive Notification**: Automated email alerts with waveform snapshots.
- **Automated Testing**: Robust test suite including E2E integration tests for alert triggering.

### Prerequisites
- Rust 1.7x (Edition 2024)
- Cargo

### Installation & Build
```bash
cd rsudp-rust
cargo build --release
```

### Usage

#### 1. Running the Receiver
```bash
./target/release/rsudp-rust --udp-port 8888 --station R6E01 --network AM
```

#### 2. Running the Streamer (Simulation)
To stream data from a MiniSEED file for testing:
```bash
./target/release/streamer --file references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --speed 1.0
```

### Testing
- **Unit Tests**: `cargo test`
- **E2E Alert Test**: `cargo test --test e2e_alert`

---

<a name="japanese"></a>
## 日本語

地震モニタリングアプリケーション `rsudp` の Rust による実装です。リアルタイム地震データの処理において、高いパフォーマンス、安定性、および信頼性を提供します。

### 主な機能
- **UDP データレシーバー**: Raspberry Shake 等からの MiniSEED または JSON データパケットを効率的に受信。
- **Pure Rust MiniSEED パーサー**: 地震データレコード用のカスタムパーサー。
- **STA/LTA アラート**: STA/LTA アルゴリズムを用いたリアルタイムのイベント検知。
- **計測震度計算**: 3成分加速度データから気象庁（JMA）計測震度を計算。
- **Web インターフェース**: モダンな WebUI (Next.js + Axum) によるリアルタイム波形表示とアラート履歴管理。
- **包括的な通知機能**: 波形スナップショットを含む自動メール通知。
- **自動テスト**: アラート発火の E2E 統合テストを含む堅牢なテストスイート。

### 動作環境
- Rust 1.7x (Edition 2024)
- Cargo

### インストールとビルド
```bash
cd rsudp-rust
cargo build --release
```

### 使い方

#### 1. レシーバーの起動
```bash
./target/release/rsudp-rust --udp-port 8888 --station R6E01 --network AM
```

#### 2. ストリーマーの起動（シミュレーション）
テスト用に MiniSEED ファイルからデータを送信する場合：
```bash
./target/release/streamer --file references/mseed/fdsnws.mseed --addr 127.0.0.1:8888 --speed 1.0
```

### テストの実行
- **ユニットテスト**: `cargo test`
- **E2E アラートテスト**: `cargo test --test e2e_alert`