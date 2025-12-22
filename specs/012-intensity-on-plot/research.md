# Research: Intensity Display on Alert Plots

## Decisions & Rationale

### 1. Font Embedding Strategy
- **Decision**: Use `include_bytes!` to bake `NotoSansJP-Bold.ttf` into the binary.
- **Rationale**: Based on the clarification, embedding ensures portability across environments (like minimal Docker containers or Raspberry Pi) without needing system-wide font installation. This aligns with the "Rigorous Testing" and "Stability" principles of the Constitution.
- **Alternatives considered**: System font lookup (rejected due to unpredictability), dynamic download (rejected due to network dependency).

### 2. Intensity Calculation Timing
- **Decision**: Calculate the max intensity over the specific 90-second snapshot window *before* calling the draw function.
- **Rationale**: To ensure the displayed intensity accurately reflects what's visible in the image, the calculation must be bounded by the same time range as the plot data.
- **Implementation**: Reuse the `JmaFilter` logic but apply it to the final sliced `Vec<f64>` data used for the plot.

### 3. JMA Color Palette
- **Decision**: Standard JMA Color Palette for Shindo levels.
- **Rationale**: Matches user expectation and provides instant recognition.
- **Palette Definition**:
  - Shindo 0: #f2f2ff (Light Grey/Blue)
  - Shindo 1: #a0eeff (Light Blue)
  - Shindo 2: #00bbff (Blue)
  - Shindo 3: #33ff00 (Green)
  - Shindo 4: #ffff00 (Yellow)
  - Shindo 5-: #ff9900 (Orange)
  - Shindo 5+: #ff2800 (Red)
  - Shindo 6-: #a50021 (Dark Red)
  - Shindo 6+: #550011 (Maroon)
  - Shindo 7: #550055 (Purple)

### 4. Layout Integration
- **Decision**: Draw the intensity box as a `plotters` element overlay in the top-right corner of the drawing area.
- **Rationale**: Allows precise positioning relative to the plot margins. The box will have a semi-transparent or solid background with white text.

## Best Practices Found

### Font Rendering in Plotters
- Use the `ab_glyph` feature of `plotters` for rendering baked-in fonts without C-library dependencies (like fontconfig), ensuring the binary remains portable.

### Multi-channel Max Intensity
- When 3 components (Z, N, E) are present, use the vector sum of filtered acceleration to calculate JMA intensity. If only one channel is present, calculate based on that channel (noting it might be lower than true JMA).
