# Research Findings: Alert Audio Playback

**Feature**: 032-alert-audio-playback
**Date**: 2026-02-05

## 1. Audio Library: `rodio`
- **Capability**: Supports MP3, WAV, OGG via `symphonia` feature flags.
- **Async Compatibility**: `rodio` operations (OutputStream creation, Sink append) are blocking.
  - **Decision**: All `rodio` calls must be wrapped in `tokio::task::spawn_blocking` to avoid stalling the `tokio` reactor core.
- **Sink Management**: `Sink::sleep_until_end()` blocks until audio finishes.
  - **Preemption**: To support preemption (stopping reset sound for new trigger), we need to hold a reference to the active `Sink` and call `stop()` or drop it when a new sound needs to play. However, since `spawn_blocking` runs in a separate thread, managing a shared mutable `Sink` across threads might be complex.
  - **Alternative**: Fire-and-forget `spawn_blocking` is simplest but doesn't easily allow stopping *from the outside*.
  - **Refined Approach**: For "Immediate Preemption", we need a shared state (e.g., `Arc<Mutex<Option<Sink>>>`) or a way to signal the playback thread.
  - **Constraint**: `rodio`'s `Sink` is `Send` but not `Sync`.
  - **Revised Decision**: Given "Fire and forget" assumption in spec vs "Immediate Preemption" requirement.
    - If we just spawn a new thread for every sound, they will mix.
    - To support preemption, we likely need a dedicated actor/task that manages the audio device and accepts commands (`Play(path)`, `Stop`).
    - **Architecture**: A dedicated `AudioController` struct held in `WebState` or passed to pipeline, protecting an `Option<Sink>` with a Mutex. When `play` is called, it drops the old `Sink` (stopping playback) and creates a new one.

## 2. Configuration Structure
- **Mapping**: `[ALERTSOUND]` section needs a flexible mapping.
- **TOML**: TOML supports tables. We can use a `[ALERTSOUND.intensity]` table.
  ```toml
  [ALERTSOUND]
  enabled = true
  trigger_file = "/path/to/trigger.mp3"
  default_reset_file = "/path/to/default.mp3"
  
  [ALERTSOUND.intensity]
  "1" = "/path/to/1.mp3"
  "5+" = "/path/to/5p.mp3"
  ```
- **Parsing**: `settings.rs` needs to parse this. `config` crate handles maps well.

## 3. Preemption Implementation Detail
- `rodio::OutputStream` needs to stay alive as long as sound is playing.
- If we re-create `OutputStream` every time, it might be slow (latency).
- Better to keep `OutputStream` and `OutputStreamHandle` alive in the `AudioController`.
- `Sink` should be recreated or cleared on new playback.
- **Concurrency**: `spawn_blocking` is for *computation*. `rodio` is *I/O*.
- Actually, `rodio` creates its own background thread for the stream. The main thread just feeds it.
- **Critical**: If we hold `Sink` in a struct, we can control it.
- **Plan**: Create `AudioController` that holds `(_stream, stream_handle)` permanently. When `play()` is called, it creates a new `Sink`, appends source, and stores the `Sink` in a `Mutex<Option<Sink>>`. Storing it overwrites the old one, which drops it, stopping the old sound. This achieves preemption naturally.

## 4. File Path Handling
- Paths can be relative or absolute.
- `std::fs::canonicalize` or resolving relative to config dir is best practice.
- `rodio` needs `File` object.

## 5. Linux ALSA
- Direct run on Linux usually works out of the box with `libasound2`.
- No extra code needed for "default device", `OutputStream::try_default()` handles it.
