# Data Model: Audio Configuration

**Feature**: 032-alert-audio-playback

## Configuration Entities (`rsudp.toml`)

### AlertSoundSettings
Maps to `[ALERTSOUND]` section.

| Field | Type | Description |
|---|---|---|
| `enabled` | `bool` | Master switch for audio playback. |
| `trigger_file` | `String` | Path to the Trigger (Warning) sound file. |
| `default_reset_file` | `String` | Path to the default Reset (Report) sound file. |
| `intensity_files` | `HashMap<String, String>` | Map of JMA intensity string (e.g. "5+") to file path. |

**Example Config**:
```toml
[ALERTSOUND]
enabled = true
trigger_file = "sounds/warning.mp3"
default_reset_file = "sounds/report_default.mp3"

[ALERTSOUND.intensity_files]
"1" = "sounds/int1.mp3"
"2" = "sounds/int2.mp3"
"3" = "sounds/int3.mp3"
"4" = "sounds/int4.mp3"
"5-" = "sounds/int5m.mp3"
"5+" = "sounds/int5p.mp3"
"6-" = "sounds/int6m.mp3"
"6+" = "sounds/int6p.mp3"
"7" = "sounds/int7.mp3"
```

## Runtime Structures

### AudioManager
Responsible for playback state.

| Field | Type | Description |
|---|---|---|
| `stream` | `OutputStream` | Handle to the audio device (must keep alive). |
| `handle` | `OutputStreamHandle` | Factory for Sinks. |
| `current_sink` | `Mutex<Option<Sink>>` | Currently playing audio. Replacing this stops previous audio. |
