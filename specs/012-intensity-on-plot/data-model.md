# Data Model: Intensity Display on Alert Plots

## Entities

### 1. ShindoLevel
Defines the mapping between instrumental intensity and visual representation.

| Field | Type | Description |
|-------|------|-------------|
| label | String | Human readable (e.g., "震度 3", "震度 5弱") |
| min_val | f64 | Minimum instrumental intensity for this level |
| bg_color | RGB | JMA standard color for the box background |
| text_color | RGB | White (#FFFFFF) for contrast |

### 2. PlotAnnotation
Internal state for the drawing engine.

| Field | Type | Description |
|-------|------|-------------|
| max_intensity | f64 | Highest calculated intensity in the window |
| shindo_class | String | Target label to draw |
| box_color | RGB | Resolved background color |

## Logic Flow

1. **Analysis**: Sliced Waveform Data (90s) -> Apply JMA Filter -> Find Peak -> Calculate Instrumental Intensity.
2. **Classification**: Instrumental Intensity -> Match `min_val` in `ShindoLevel` map.
3. **Rendering**:
   - Draw Waveforms and Spectrograms.
   - Calculate Bounding Box for top-right corner.
   - Fill Rectangle with `bg_color`.
   - Render `label` using `Noto Sans Japanese` in white.
