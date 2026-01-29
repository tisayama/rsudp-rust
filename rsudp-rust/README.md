# rsudp-rust

[English](#english) | [日本語](#japanese)

---

<a name="english"></a>
## English

Rust implementation of the `rsudp` seismic monitoring application. It provides high performance, stability, and reliability for real-time seismic data processing, specifically designed for Raspberry Shake 4D and compatible devices.

### Key Features

*   **UDP Data Receiver**:
    *   Efficiently receives MiniSEED or JSON data packets via UDP.
    *   Supports high-throughput data streams with minimal latency using Tokio's asynchronous runtime.
*   **Pure Rust MiniSEED Parser**:
    *   Custom-built, safe, and fast parser for seismic data records (MiniSEED format).
    *   Handles various encoding formats and block sizes automatically.
*   **STA/LTA Alerting**:
    *   Real-time event detection using the classic Short-Term Average / Long-Term Average (STA/LTA) algorithm.
    *   Configurable thresholds and window sizes for flexible triggering.
*   **JMA Seismic Intensity Calculation**:
    *   Calculates the **JMA (Japan Meteorological Agency) Seismic Intensity Scale** in real-time.
    *   Uses 3-component acceleration data (ENE, ENN, ENZ) with rigorous signal processing.
*   **Web Interface**:
    *   Modern, responsive WebUI built with **Next.js** (Frontend) and **Axum** (Backend).
    *   Displays real-time waveform plots, calculated intensity, and alert history.
*   **Comprehensive Notification**:
    *   Automated email alerts containing event details and waveform snapshots (PNG).
    *   Supports integration with external SNS or messaging platforms.
*   **Automated Testing**:
    *   Includes a robust test suite with Unit Tests and End-to-End (E2E) integration tests to ensure reliability.

### Engineering Documentation: JMA Seismic Intensity Calculation

This program calculates the JMA Seismic Intensity based on the official "Method of Calculating Seismic Intensity" defined by the JMA, with specific engineering optimizations for real-time digital signal processing.

#### 1. Calculation Process

| Process | Description | File |
| :--- | :--- | :--- |
| **Physical Conversion** | Converts digital sensor output (Counts) to physical acceleration (Gal = cm/s²) using station metadata. | `src/intensity/calc.rs` |
| **Time Alignment** | Synchronizes 3-component packets with potentially different arrival times into a continuous buffer with 10ms precision (at 100Hz). | `src/intensity/calc.rs` |
| **Preprocessing (Detrend)** | Removes linear trends and DC offsets (e.g., gravity on vertical components) using least-squares linear regression. | `src/intensity/filter.rs` |
| **Window Function (Tapering)** | Applies a Cosine Taper to the 5% edges of the window to suppress FFT artifacts (see below). | `src/intensity/filter.rs` |
| **Frequency Filtering** | Applies the three official JMA filters (Period-effect, High-cut, Low-cut) in the frequency domain after FFT. | `src/intensity/filter.rs` |
| **Intensity Determination** | Calculates the vector magnitude $a(t) = \sqrt{x^2+y^2+z^2}$ after Inverse FFT. The intensity is derived from the acceleration value $a$ that persists for a cumulative total of at least 0.3 seconds, using $I = 2 \log_{10}(a) + 0.94$. | `src/intensity/filter.rs` |

#### 2. Importance of Cosine Taper (Window Function)

In this implementation, a **Cosine Taper** is applied to the 5% edges (10% total) of the 60-second calculation window before performing FFT.

*   **Why is it necessary?**
    FFT assumes that the input signal is periodic (infinite repetition). However, real-world seismic data often contains DC offsets (especially the ~$1g$ gravity component on the Z-axis) or low-frequency drift. If the values at the start and end of the window do not match, it creates a sharp discontinuity (step) when treated as a periodic signal. This results in **Spectral Leakage**, generating false noise across all frequencies.
*   **Impact on Intensity**:
    Since the JMA filter heavily amplifies low frequencies (below 0.5Hz), this leakage manifests as a massive low-frequency "swell" or spike after Inverse FFT, causing high false intensity readings even during background noise.
*   **Solution**:
    Tapering smoothly reduces the signal amplitude to zero at the window boundaries, eliminating discontinuities and ensuring numerical stability.

#### 3. Differences from the Official Specification

1.  **Signal Preprocessing**:
    *   While the official spec assumes an ideal continuous signal, this digital implementation requires **Tapering** and **Linear Detrending** to handle finite windows and discrete sampling correctly.
2.  **Calculation Interval**:
    *   To provide real-time monitoring, this program calculates intensity using a **Sliding Window** that advances every 1 second, whereas the official spec does not mandate a specific interval.
3.  **Input Channels**:
    *   Calculation strictly requires acceleration channels (ENE, ENN, ENZ). Velocity channels (EHZ) are not supported for intensity calculation in this version to avoiding differentiation noise.

### Prerequisites
- **Rust**: Version 1.7x or later (Edition 2024 recommended).
- **Cargo**: Included with Rust.
- **Dependencies**: `build-essential` (for Make and C compiler).

### Installation & Build

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-org/rsudp-rust.git
   cd rsudp-rust
   ```

2. **Build**:
   ```bash
   make
   ```

3. **Install to system (requires sudo)**:
   This creates the `rsudp` user, installs the binary to `/usr/local/bin`, and sets up the systemd service.
   ```bash
   sudo make install
   ```

4. **Configuration**:
   Edit the configuration file:
   ```bash
   sudo nano /etc/rsudp/rsudp.toml
   ```

### Service Management

- **Start/Enable Service**:
  ```bash
  sudo systemctl start rsudp
  sudo systemctl enable rsudp
  ```

- **Check Status**:
  ```bash
  sudo systemctl status rsudp
  ```

- **View Logs**:
  ```bash
  sudo journalctl -u rsudp -f
  ```

### Usage (Manual Mode)

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

地震モニタリングアプリケーション `rsudp` の Rust による実装です。Raspberry Shake 4D 等のデバイス向けに設計されており、リアルタイム地震データ処理において高いパフォーマンス、安定性、および信頼性を提供します。

### 主な機能

*   **UDP データレシーバー**:
    *   MiniSEED または JSON 形式のデータパケットを UDP 経由で効率的に受信します。
    *   Tokio 非同期ランタイムにより、低遅延かつ高スループットなデータ処理を実現しています。
*   **Pure Rust MiniSEED パーサー**:
    *   地震データレコード（MiniSEED形式）のための、安全かつ高速な自作パーサーを搭載しています。
    *   多様なエンコーディングやブロックサイズを自動的に処理します。
*   **STA/LTA アラート**:
    *   古典的な STA/LTA（Short-Term Average / Long-Term Average）アルゴリズムを用いた、リアルタイムのイベント検知機能です。
    *   しきい値やウィンドウサイズの設定により、柔軟なトリガー検知が可能です。
*   **気象庁（JMA）計測震度計算**:
    *   3成分加速度データ（ENE, ENN, ENZ）から、**気象庁計測震度**をリアルタイムに計算します。
    *   厳密な信号処理に基づき、実際の揺れを正確に数値化します。
*   **Web インターフェース**:
    *   **Next.js** (フロントエンド) と **Axum** (バックエンド) で構築されたモダンな WebUI です。
    *   リアルタイムの波形表示、計算された震度、および過去のアラート履歴を閲覧できます。
*   **包括的な通知機能**:
    *   イベント詳細と波形スナップショット（PNG画像）を含むアラートメールを自動送信します。
    *   外部 SNS やメッセージングプラットフォームとの連携もサポート可能です。
*   **自動テスト**:
    *   信頼性を担保するため、単体テスト（Unit Tests）および E2E 統合テスト（Integration Tests）を含む堅牢なテストスイートを備えています。

### 計測震度計算の理論と実装 (Engineering Documentation)

本プログラムにおける気象庁（JMA）計測震度の計算プロセスを解説します。本実装は、気象庁が告示する「計測震度の算出方法」に基づきつつ、デジタル信号処理における数値的安定性を確保するための工学的最適化を加えています。

#### 1. 計算プロセスと担当ファイル

| プロセス | 内容 | 担当ファイル |
| :--- | :--- | :--- |
| **物理量変換** | 加速度センサー（EN*）のデジタル出力（Counts）を物理量（Gal = cm/s²）に変換します。 | `src/intensity/calc.rs` |
| **時間軸アライメント** | 到着時刻の異なる3成分パケットを同期させ、10ms（100Hz時）精度の連続バッファを生成します。 | `src/intensity/calc.rs` |
| **前処理 (Detrend)** | 最小二乗法により窓内の線形トレンド（重力オフセットやドリフト）を除去します。 | `src/intensity/filter.rs` |
| **窓関数 (Tapering)** | FFT時のアーティファクトを抑制するため、窓の両端にCosine Taperを適用します（詳細は後述）。 | `src/intensity/filter.rs` |
| **周波数領域フィルタ** | FFT後、JMA定義の3つのフィルタ（周期効果、ハイカット、ローカット）を乗算します。 | `src/intensity/filter.rs` |
| **震度の決定** | 逆FFT後のベクトル合成加速度波形 $a(t)$ から、「加速度 $a$ 以上となる時間の合計が0.3秒」を満たす最大の $a$ を用い、公式 $I = 2 \log_{10}(a) + 0.94$ で算出します。 | `src/intensity/filter.rs` |

#### 2. Cosine Taper（窓関数）の重要性

本プログラムでは、FFT（高速フーリエ変換）を行う直前に、60秒の計算窓の両端5%（計10%）に対して **Cosine Taper** を適用しています。

*   **なぜ必要なのか？**
    FFTは「入力波形が無限に繰り返される（周期性がある）」ことを前提としたアルゴリズムです。しかし、実際の観測データにはDCオフセット（特にENZ成分の重力加速度 $1g$ 等）やドリフトが含まれており、切り出した窓の「開始点」と「終了点」の値が一致しません。この不連続点は、FFTにおいて「インパルス的な段差」として処理され、全周波数帯域に広がる偽のノイズ（**スペクトル漏洩 / Spectral Leakage**）を発生させます。
*   **JMAフィルタへの影響**:
    JMA震度計算フィルタは低域（0.5Hz以下）に大きな利得を持つため、このスペクトル漏洩による低域ノイズが逆FFT後に「巨大なうねり（スパイク）」として現れ、無震時にもかかわらず高い震度を算出してしまう原因となります。
*   **解決策**:
    Cosine Taperを適用することで、窓の両端付近の振幅を滑らかに 0 へ収束させます。これにより境界の不連続性が排除され、数値演算上のアーティファクトによる誤検知を劇的に抑制しています。

#### 3. 気象庁公式仕様との相違点

本プログラムは実用上の理由から、公式仕様と以下の点で挙動が異なります。

1.  **信号処理上の前処理**:
    *   公式仕様は理想的な連続信号を想定していますが、コンピュータによる有限窓の処理では上記の **Tapering** や **Detrend** が不可欠です。これらは理論値を歪めるためのものではなく、理論値を正しく計算するための「デジタル信号処理の標準的な作法」です。
2.  **計算間隔**:
    *   本プログラムはリアルタイム監視のため、1秒ごとにスライドする移動窓で計算を行っています。
3.  **入力チャンネル**:
    *   震度計算は加速度センサー（ENE, ENN, ENZ）からの入力を前提としています。

### 動作環境
- Rust 1.7x (Edition 2024)
- Cargo
- 依存パッケージ: `build-essential`

### インストールとビルド

1. **リポジトリのクローン**:
   ```bash
   git clone https://github.com/your-org/rsudp-rust.git
   cd rsudp-rust
   ```

2. **ビルド**:
   ```bash
   make
   ```

3. **システムへのインストール (sudo必須)**:
   `rsudp` ユーザーの作成、バイナリの配置、systemdサービスの登録を行います。
   ```bash
   sudo make install
   ```

4. **設定**:
   ステーション情報などを編集します。
   ```bash
   sudo nano /etc/rsudp/rsudp.toml
   ```

### サービスの管理

- **サービスの開始・自動起動有効化**:
  ```bash
  sudo systemctl start rsudp
  sudo systemctl enable rsudp
  ```

- **状態確認**:
  ```bash
  sudo systemctl status rsudp
  ```

- **ログ確認**:
  ```bash
  sudo journalctl -u rsudp -f
  ```

### 使い方（手動実行）

#### 1. レシーバーの起動（ライブモード）
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
