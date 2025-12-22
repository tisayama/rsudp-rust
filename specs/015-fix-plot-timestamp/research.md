# Research: rsudp Plot Timestamp & Grid Fix

## Decisions & Rationale

### 1. X-axis Timestamp Formatting
- **Decision**: Use `plotters`'s `x_label_formatter` to convert relative seconds (f64) into UTC `HH:MM:SS` strings.
- **Rationale**: The current plotting engine uses `f64` ranges for simplicity. Instead of changing the coordinate system to `DateTime` (which is complex in `plotters`), we can simply format the labels by adding the relative offset to the `start_time`.
- **Implementation**: `format!("{}", (start_time + Duration::milliseconds((x * 1000.0) as i64)).format("%H:%M:%S"))`.

### 2. Disabling Grid Lines for Waveforms
- **Decision**: Use `.disable_mesh()` on the `waveform_area` chart configuration.
- **Rationale**: The user requested to match `rsudp`'s look, which typically has a clean background without grid lines for seismic waveforms to improve signal clarity.
- **Implementation**: `chart.configure_mesh().disable_mesh().draw()?`.

### 3. Axis Label Consolidation
- **Decision**: Display X-axis labels and descriptors only on the bottom-most chart (Spectrogram) and hide them from the upper chart (Waveform).
- **Rationale**: This is a standard `rsudp` / `matplotlib` optimization when stacking plots with a shared X-axis. It reduces visual clutter and maximizes the vertical space for the actual data.
- **Implementation**: Call `.disable_x_axis()` or simply don't configure labels for the upper chart.

### 4. Grid Display for Spectrogram
- **Decision**: Keep spectrogram grid disabled as well, or use very light lines if `rsudp` does so.
- **Rationale**: Usually, heatmap-style plots like spectrograms don't use grids in `rsudp` to avoid distracting from the color intensity.

## Best Practices Found

### Coordinate Alignment
- Ensure that the `build_cartesian_2d` range for both the Waveform and Spectrogram is identical (`0.0..total_seconds`) so that the vertical alignment of events is preserved when labels are only shown on the bottom.
