# Research: rsudp Plot Compatibility

## Decisions & Rationale

### 1. Spectrogram Calculation
- **Decision**: Use `rustfft` for FFT computation with a manual sliding window implementation.
- **Rationale**: `rsudp` uses Matplotlib's `specgram`, which defaults to `NFFT=256`, `noverlap=128`, and a Hanning window. While Rust has FFT libraries, there is no direct equivalent to `specgram` that handles the windowing and overlap out-of-the-box in the same way. Implementing this manually ensures we can match `rsudp`'s output exactly.
- **Alternatives considered**: `spectrum` crate (less flexible for custom overlap/windowing matching).

### 2. Colormap
- **Decision**: Implement the `viridis` (or `rsudp`'s specific) colormap manually or use `scarlet` / `palette`.
- **Rationale**: `rsudp` typically uses `viridis` or `magma`. We need to map the normalized power spectral density (PSD) values to RGB colors to generate the spectrogram bitmap. `plotters` supports custom color maps, but we might need to define the gradient explicitly to match Matplotlib's look.

### 3. Layout Composition
- **Decision**: Use `plotters`'s `BitMapBackend` with multiple drawing areas.
- **Rationale**: `plotters` allows splitting a drawing area into vertical chunks. We can create a layout where the top 70% is the waveform and the bottom 30% is the spectrogram (or 50/50), mimicking `rsudp`'s vertical stack.
- **Alternatives considered**: Generating two separate images and stitching them (less efficient).

### 4. Font and Style
- **Decision**: Use standard sans-serif fonts available in `plotters`.
- **Rationale**: `rsudp` uses Matplotlib's default font (usually DejaVu Sans). `plotters` default font is close enough for "compatibility" without needing to bundle custom font files, provided the sizes and weights are matched.

## Best Practices Found

### Spectrogram Normalization
- Matplotlib's `specgram` returns $10 \log_{10}(P_{xx})$ (dB) by default for visualization. We must replicate this logarithmic scaling and dynamic range clipping to get the same visual "heat" map.

### Data Alignment
- The spectrogram time axis must align perfectly with the waveform time axis. This requires careful calculation of the time lag introduced by the windowing function (usually centered or edge-aligned).
