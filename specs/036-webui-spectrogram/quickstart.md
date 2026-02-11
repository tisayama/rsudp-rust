# Quickstart: WebUI Spectrogram & rsudp-Compatible Plot

**Phase 1 Output** | **Date**: 2026-02-10

## Prerequisites

- Rust toolchain (1.7x stable)
- Node.js 22+ / npm
- rsudp-rust built and runnable
- MiniSEED test data available (references/rsudp/rsudp/test/testdata.ms)

## Setup

### 1. Install frontend dependencies

```bash
cd webui
npm install
```

### 2. Build backend

```bash
cd rsudp-rust
cargo build
```

### 3. Generate Inferno Colormap LUT

```bash
python3 -c "
import matplotlib.pyplot as plt
import json
cmap = plt.get_cmap('inferno')
table = [[int(c*255) for c in cmap(i/255)[:3]] for i in range(256)]
print('export const INFERNO_RGB: [number, number, number][] = ' + json.dumps(table) + ';')
" > webui/lib/inferno-colormap.ts
```

Or use the pre-generated file that will be included in the implementation.

## Development Workflow

### Start backend (with streamer simulation)

```bash
cd rsudp-rust
cargo run -- --config ../rsudp.toml --simulate ../references/rsudp/rsudp/test/testdata.ms
```

### Start frontend

```bash
cd webui
npm run dev
```

### Access WebUI

Open `http://localhost:3000` in a desktop browser.

## Verification Checklist

1. **Dark theme**: Page background is #202530, text is light gray
2. **Waveform + Spectrogram**: Each channel shows waveform (top 2/3) + spectrogram (bottom 1/3)
3. **Channel sorting**: Z-ending channels appear first
4. **Inferno colormap**: Spectrogram uses purple-to-yellow gradient
5. **Event markers**: Blue dashed (trigger) and red dashed (reset) lines on both panels
6. **Title**: "{STATION} Live Data - Detected Events: N" centered at top
7. **rsudp-rust branding**: "rsudp-rust" text in top-left corner
8. **Intensity badge**: Appears on trigger, shows JMA color, disappears 30s after reset
9. **Backfill**: Refresh page → data immediately visible (no waiting for live data)
10. **Spectrogram controls**: Toggle, frequency range, log Y-axis in control panel

## Testing

### Frontend unit tests

```bash
cd webui
npm test
```

Tests cover:
- `SpectrogramRenderer.test.ts`: u8→ImageData変換正確性（LUTマッピング、列スクロール動作）
- `inferno-colormap.test.ts`: LUT boundary values (index 0, 127, 255)
- `engineering-format.test.ts`: SI prefix formatting (1000 → "1k", 0.001 → "1m")

### Backend tests

```bash
cd rsudp-rust
cargo test
```

Existing tests should continue to pass. New tests:
- `compute_spectrogram_u8`: u8正規化の境界値テスト（0, 255, 中間値）
- Backfill protocol: Spectrogram + Waveformパケット送信の統合テスト

## Key Files to Edit

| File | Change |
|------|--------|
| `rsudp-rust/src/web/plot.rs` | MODIFY: `compute_spectrogram_u8()`追加、`SpectrogramU8`構造体追加 |
| `rsudp-rust/src/web/stream.rs` | MODIFY: backfill handler, spectrogram streaming, buffer sync |
| `webui/lib/SpectrogramRenderer.ts` | NEW: u8→ImageData描画エンジン（FFT計算なし） |
| `webui/lib/inferno-colormap.ts` | NEW: 256-entry color LUT |
| `webui/lib/engineering-format.ts` | NEW: SI prefix formatter |
| `webui/components/ChannelPairCanvas.tsx` | NEW: waveform + spectrogram paired renderer |
| `webui/components/IntensityBadge.tsx` | NEW: JMA intensity badge |
| `webui/src/app/page.tsx` | MODIFY: dark theme, layout, sorting |
| `webui/src/app/globals.css` | MODIFY: dark theme base |
| `webui/components/ControlPanel.tsx` | MODIFY: spectrogram controls |
| `webui/hooks/useWebSocket.ts` | MODIFY: backfill protocol + spectrogram packet parsing |
| `webui/lib/types.ts` | MODIFY: new type definitions |
