# Implementation Plan: WebUI Spectrogram & rsudp-Compatible Plot

**Branch**: `036-webui-spectrogram` | **Date**: 2026-02-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/036-webui-spectrogram/spec.md`

## Summary

rsudpのPlot機能（波形＋スペクトログラムのペア表示）をWebUIで再現する。既存のCanvas 2Dベースの波形レンダリングを拡張し、サーバーサイドFFT（既存の`rustfft`/`compute_spectrogram()`を活用）によるリアルタイムスペクトログラム表示、ダークテーマ化、チャンネルソート、イベントマーカー（トリガー/リセット）、震度階級バッジ表示、WebSocket再接続時のバックフィルを実装する。サーバーがu8正規化済みスペクトログラムデータをWebSocket経由で配信し、クライアントはInferno LUT参照による描画のみを行う軽量アーキテクチャ。

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2021) + TypeScript 5 / Next.js 14.2
**Primary Dependencies**:
- Backend: `axum` 0.7 (WebSocket), `tokio` 1.0 (async), `rustfft` 6.2 (既存), `serde_json` (serialization)
- Frontend: `react` 18, `next` 14.2, `tailwindcss` 3.4 (FFT依存なし — サーバーサイド計算)
**Storage**: In-memory ring buffers (server-side `VecDeque`, client-side `Float32Array`)
**Testing**: `cargo test` (Rust), `jest` 30.2 (TypeScript)
**Target Platform**: Desktop browser (Chrome/Firefox/Safari), Linux server
**Project Type**: Web application (Rust backend + Next.js frontend)
**Performance Goals**: 15+ FPS for 4 channels × (waveform + spectrogram), 1s backfill render
**Constraints**: 表示ウィンドウ最大300秒、Canvas 2D API使用、サーバーサイドFFT（既存rustfft）、フロントエンドFFT依存なし
**Scale/Scope**: 1-4チャンネル同時表示、100 Hz サンプルレート

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. 安定性と信頼性 | ✅ PASS | バックフィル機能で接続切断時のデータ復旧を保証。エッジケース（空データ、短ウィンドウ等）を仕様で定義済み |
| II. 厳密なテスト | ✅ PASS | 既存compute_spectrogram()のテスト活用、u8正規化テスト、バックフィルプロトコルの統合テスト、クライアント描画テストを計画 |
| III. 高いパフォーマンス | ✅ PASS | サーバーサイドrustfftによる高速FFT、u8データ転送で帯域最小化、ImageData+drawImage(self)によるGPU活用スクロール描画、15+ FPS目標 |
| IV. コードの明瞭性と保守性 | ✅ PASS | 既存FFTコード再利用でコード重複回避、ChannelPairRenderer等の明確なコンポーネント分割 |
| V. 日本語による仕様策定 | ✅ PASS | 仕様書は日本語Q&Aで策定済み |
| VI. 標準技術スタック | ✅ PASS | Next.js + Tailwind CSS + Rust WebSocket — 規定通り |
| VII. 自己検証の義務 | ✅ PASS | streamerシミュレーションによる動作確認を計画 |
| VIII. ブランチ運用 | ✅ PASS | `036-webui-spectrogram`ブランチで開発 |

## Project Structure

### Documentation (this feature)

```text
specs/036-webui-spectrogram/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── websocket-protocol.md
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── references/          # rsudp screenshots
    ├── rsudp_single_channel.png
    ├── rsudp_4channel.png
    ├── rsudp_alert_markers.png
    └── rsudp_spectrogram_scaling.png
```

### Source Code (repository root)

```text
rsudp-rust/src/
├── web/
│   ├── plot.rs          # MODIFY: u8正規化スペクトログラム計算関数追加（既存compute_spectrogram()を活用）
│   ├── stream.rs        # MODIFY: backfill protocol, spectrogram streaming, buffer size sync
│   ├── routes.rs        # MODIFY: add /api/channels endpoint if missing
│   └── ...              # Other web modules (unchanged)
└── ...                  # Other modules (unchanged)

webui/
├── src/app/
│   ├── page.tsx         # MODIFY: dark theme, channel sorting, layout restructure
│   ├── layout.tsx       # MODIFY: dark theme globals
│   └── globals.css      # MODIFY: dark theme base styles
├── components/
│   ├── WaveformCanvas.tsx        # REPLACE: → ChannelPairCanvas.tsx
│   ├── ChannelPairCanvas.tsx     # NEW: waveform + spectrogram paired rendering (描画のみ、FFTなし)
│   ├── ControlPanel.tsx          # MODIFY: dark theme + spectrogram controls
│   ├── IntensityBadge.tsx        # NEW: seismic intensity class badge
│   ├── AlertSettingsPanel.tsx    # MODIFY: dark theme
│   └── PerformanceMonitor.tsx    # MODIFY: dark theme
├── hooks/
│   ├── useWebSocket.ts           # MODIFY: backfill protocol + spectrogram packet parsing
│   └── useAlerts.ts              # (unchanged)
├── lib/
│   ├── types.ts                  # MODIFY: add spectrogram types
│   ├── RingBuffer.ts             # (unchanged)
│   ├── SpectrogramRenderer.ts    # NEW: u8データ→ImageData描画エンジン（FFT計算なし）
│   ├── inferno-colormap.ts       # NEW: 256-entry inferno LUT
│   ├── engineering-format.ts     # NEW: SI prefix formatter
│   └── RingBuffer.test.ts        # (unchanged)
└── __tests__/
    ├── SpectrogramRenderer.test.ts # NEW: u8→ImageData変換テスト
    ├── inferno-colormap.test.ts    # NEW: colormap boundary tests
    └── engineering-format.test.ts  # NEW: formatter tests
```

**Structure Decision**: サーバーサイドFFTアーキテクチャ。既存の`compute_spectrogram()`（plot.rs）を活用してサーバーでFFT計算→u8正規化し、WebSocket経由でスペクトログラムデータを配信。フロントエンドはFFT依存なし（fft.jsは不要）、Inferno LUT参照によるImageData描画のみの軽量構成。
