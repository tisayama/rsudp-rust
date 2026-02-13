# API Contract: Settings Endpoint (Extended)

**Endpoint**: `GET /api/settings` and `POST /api/settings`
**Change**: Add bandpass filter fields to the existing PlotSettings response

## GET /api/settings

**Response** (JSON):
```json
{
  "scale": 1.0,
  "window_seconds": 90.0,
  "save_pct": 0.7,
  "output_dir": "/path/to/output",
  "deconvolve": true,
  "units": "CHAN",
  "eq_screenshots": false,
  "filter_waveform": false,
  "filter_highpass": 0.7,
  "filter_lowpass": 2.0
}
```

### New Fields

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `filter_waveform` | bool | Whether bandpass filter is active for waveform display | `false` |
| `filter_highpass` | f64 | High-pass cutoff frequency in Hz | `0.7` |
| `filter_lowpass` | f64 | Low-pass cutoff frequency in Hz | `2.0` |

### Notes

- These are read-only in practice (sourced from `rsudp.toml` `[plot]` section)
- The frontend uses these values only for display labels ("Bandpass (X - Y Hz)")
- The `POST /api/settings` endpoint will accept and store these fields but they do not affect backend signal processing
- When `filter_waveform` is `false`, the frontend hides the "Bandpass" label entirely
