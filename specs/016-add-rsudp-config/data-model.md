# Data Model: Configuration Schema

## Entities

### `Settings` (Root)
The primary configuration object.

| Field | Type | Description |
|-------|------|-------------|
| settings | SettingsSection | Global application settings |
| plot | PlotSettings | Waveform and Spectrogram plotting |
| alert | AlertSettings | STA/LTA and triggering logic |
| forward | ForwardSettings | Data forwarding (UDP) |
| notifications | NotificationSettings | Various alert providers |
| rsam | RsamSettings | RSAM calculation and forwarding |

### `SettingsSection` (Sub-entity)
| Field | Type | Default | Validation |
|-------|------|---------|------------|
| port | u16 | 8888 | 1-65535 |
| station | String | "Z0000" | Max 5 chars |
| output_dir | PathBuf | "~/rsudp" | Must be writable |
| debug | bool | true | |

### `PlotSettings` (Sub-entity)
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| enabled | bool | true | |
| duration | u32 | 90 | Seconds to plot |
| spectrogram | bool | true | Show spectrogram |
| filter_highpass | f64 | 0.7 | Hz |
| filter_lowpass | f64 | 2.0 | Hz |

### `AlertSettings` (Sub-entity)
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| enabled | bool | true | |
| sta | f64 | 6.0 | Seconds |
| lta | f64 | 30.0 | Seconds |
| threshold | f64 | 3.95 | Trigger ratio |
| deconvolve | bool | false | |
| units | String | "VEL" | VEL, ACC, or DISP |

## Merging & Priority Rules

1. **Default Values**: Hardcoded in `src/settings.rs` via `impl Default`.
2. **Config File**: Loaded from `~/.rsudp/settings.{toml,yaml}`. TOML wins if both exist.
3. **Environment Variables**: Prefixed with `RUSTRSUDP_`.
4. **CLI Arguments**: Highest priority. Overwrites everything else.
