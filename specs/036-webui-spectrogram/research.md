# Research: WebUI Spectrogram & rsudp-Compatible Plot

**Phase 0 Output** | **Date**: 2026-02-10

## R-001: FFT Computation Strategy

**Decision**: サーバーサイドFFT（既存の`rustfft`を利用）+ u8正規化済みスペクトログラムデータのWebSocket配信

**Rationale**:
- `rsudp-rust/src/web/plot.rs`に既存の`compute_spectrogram()`関数がある（rustfft 6.2使用、テスト済み）
- Hanning窓、DC除去（mean subtraction）、PSD計算が実装済み — 新規FFT実装不要
- サーバーでpower scaling (`mag^(1/10)`) + 正規化 (0-255 u8) を行い、クライアントはInferno LUT参照のみで描画可能
- フロントエンドのFFT依存（fft.js）を完全に排除 — ビルドサイズ削減、ブラウザCPU負荷軽減
- 帯域影響: 90秒/4チャンネルのバックフィル時で約844 KB（波形のみ144 KBの約6倍）— 十分に許容範囲
- ライブストリーミング時の追加帯域: チャンネルあたり約0.78 KB/秒（100Hz, NFFT=128, overlap=90%で約7.8列/秒 × 65 bins × u8）

**Alternatives considered**:
- `fft.js`（ブラウザFFT）: 動作するが、既にRust側に同等実装があるため重複。新規npm依存の追加も不要に
- Web Audio API AnalyserNode: 音声ストリーム用で、WebSocketからの任意データには不向き
- サーバーでf64生データ送信: 帯域が8倍に増加し不要。u8正規化で視覚的に十分な精度（256段階）

## R-002: Spectrogram Canvas Rendering Strategy

**Decision**: サーバーからのu8列データをInferno LUT参照でImageDataに変換 + drawImage(self)スクロール方式

**Rationale**:
- サーバーから受信した各スペクトログラム列（u8[] × frequencyBins）をInferno LUT参照でRGBAに変換
- 1列（1 × height ピクセル）のImageDataとしてCanvasの右端にputImageData
- 既存コンテンツはdrawImage(canvas, 1, 0, w-1, h, 0, 0, w-1, h)で1ピクセル左にシフト — GPU加速で高速
- クライアントはFFT計算不要、LUT参照+ImageData書き込みのみで超軽量

**Alternatives considered**:
- クライアントサイドFFT + 描画: 動作するが既存サーバーFFT実装と重複
- fillRect per pixel: 154,200 draw calls/frame — パフォーマンス不足
- WebGL: Canvas 2D仕様（FR-013）に反する

## R-003: Inferno Colormap Implementation

**Decision**: Pythonで生成した256エントリのRGBルックアップテーブルをTypeScript定数として埋め込み

**Rationale**:
- matplotlibのinferno colormapと完全一致を保証
- ランタイム補間不要（0コスト参照）
- Uint8Array(256*4) = 1KBのフラットRGBAテーブルで最速アクセス
- ソースコード ~3KB

**Alternatives considered**:
- ランタイム色補間: アンカーポイントからの計算は誤差が生じうる
- 外部ライブラリ（d3-scale-chromatic等）: 不要な依存追加

## R-004: WebSocket Backfill Protocol

**Decision**: WebSocket接続時にクライアントからJSON requestを送信、サーバーがバイナリWaveform＋Spectrogramパケットのバッチで応答

**Rationale**:
- 既存のバイナリWaveformパケット形式を再利用し、新たにSpectrogramパケット（u8正規化済み）を追加
- バックフィル時はChannelBufferから波形データ抽出 → compute_spectrogram() → 正規化 → 両方送信
- サーバーのChannelBufferのextract_window()メソッドが既に時間ベースのデータ抽出をサポート
- REST APIよりWebSocket内で完結する方がシンプル（追加エンドポイント不要）

**Alternatives considered**:
- REST API `/api/backfill?since=timestamp`: 別経路でのデータ取得は複雑化
- サーバーが自動的にバッファ全体を送信: クライアントの「最後のタイムスタンプ」情報がないと重複発生
- 波形のみ送信しクライアントでFFT: 既存サーバーFFTと重複、クライアント負荷増

## R-005: Dark Theme Migration Strategy

**Decision**: Tailwind CSS のカスタムカラーパレット + globals.css の変更で全ページダーク化

**Rationale**:
- tailwind.config.tsにカスタムカラー（bg: '#202530', fg: 'rgba(204,204,204,1.0)', line: '#c28285'）を追加
- 各コンポーネントのクラス名を一括更新（bg-white → bg-[#202530], text-slate-700 → text-gray-300等）
- 既存のライトテーマクラスを単純に置換するだけで、構造変更は最小限

**Alternatives considered**:
- CSS変数によるテーマ切替: 仕様でライトテーマ廃止が確定しているため不要
- styled-components: 既存Tailwind構成に反する

## R-006: Engineering Notation Formatter

**Decision**: カスタム軽量フォーマッタをTypeScriptで実装

**Rationale**:
- SI接頭辞（μ, m, k, M等）で数値を表示する関数は20-30行で実装可能
- 外部ライブラリ不要
- rsudpのEngFormatter互換の出力形式を保証

**Alternatives considered**:
- `Intl.NumberFormat`: SI接頭辞をサポートしない
- `d3-format`: 不要な依存追加
