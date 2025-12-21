# Data Model: rsudp Plot Compatibility

## Entities

### 1. SpectrogramConfig
Configuration for spectrogram generation, matching `rsudp` defaults.

| Field | Type | Description |
|-------|------|-------------|
| nfft | usize | FFT window size (default: 256) |
| noverlap | usize | Overlap between windows (default: 128) |
| sampling_rate | f64 | Sample rate of the input data (Hz) |
| colormap | String | Name of the colormap (e.g., "viridis") |

### 2. PlotLayout
Configuration for the combined image layout.

| Field | Type | Description |
|-------|------|-------------|
| width | u32 | Image width in pixels |
| height | u32 | Image height in pixels |
| waveform_ratio | f64 | Portion of height for waveform (e.g., 0.6) |
| spectrogram_ratio | f64 | Portion of height for spectrogram (e.g., 0.4) |

## State Transitions

### Image Generation Pipeline
1. **Input**: Receive 60s waveform buffer (`Vec<f64>`) and metadata.
2. **Preprocessing**: Detrend and demean waveform (if needed).
3. **FFT**: Apply sliding window (Hanning), compute FFT, calculate Power Spectral Density (PSD).
4. **Rendering**:
   - Create `BitMapBackend`.
   - Split area into top (Waveform) and bottom (Spectrogram).
   - Draw Waveform with time axis.
   - Draw Spectrogram heatmap aligned to time axis.
5. **Output**: Save PNG to `alerts/` directory.
