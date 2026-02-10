# Specification: Alert Audio Playback

**Feature**: Alert Audio Playback
**Status**: DRAFT
**Feature Branch**: `032-alert-audio-playback`

## 1. Executive Summary

This feature implements server-side audio playback for alerts in the Rust application, replicating the functionality of the original `rsudp`. It enables the system to play specific sound files (MP3, etc.) when an alert is triggered (Trigger) and when it is reset (Reset). Crucially, the sound played during the Reset phase dynamically changes based on the maximum seismic intensity observed during the event. This audio feedback provides immediate auditory situational awareness to users in the physical vicinity of the server (e.g., Raspberry Pi with speakers).

## 2. Clarifications

### Session 2026-02-05
- Q: What happens if a specific JMA intensity sound is not configured? → A: Use a common default reset sound and output an error log.
- Q: Should the system support selecting a specific audio output device? → A: No, use the OS default output device.
- **Trigger Sound Behavior**: Play Once. The trigger sound plays a single time upon alert activation and does not loop.
*   **Concurrency (Preemption)**: **Immediate Preemption**. If a new Trigger event occurs while a Reset (Intensity Report) sound is playing, the Reset sound MUST stop immediately, and the new Trigger sound MUST start.
*   **Assets**: User Provided Only. The system will not ship with default audio files. Users must provide valid paths in the configuration. If a file is missing, the system logs an error and continues.

## 3. User Scenarios

### 3.1. Earthquake Alert Sequence
**Actor**: User (listener)
**Scenario**: A seismic event occurs and triggers an alert.
**Flow**:
1. The system detects an STA/LTA trigger.
2. The system immediately plays the configured **Trigger Sound** via the server's speakers.
3. The shaking continues; the system calculates the seismic intensity.
4. The alert condition ends (Reset).
5. The system determines the maximum JMA Seismic Intensity reached during the event.
6. The system plays the **Reset Sound** corresponding to that specific intensity class.
7. The user hears the intensity report sound and understands the severity of the event.

### 3.2. Configuration
**Actor**: User (Administrator)
**Scenario**: Configuring custom alert sounds.
**Flow**:
1. User prepares MP3 files for the trigger and for each intensity level.
2. User edits `rsudp.toml` under `[ALERTSOUND]`.
3. User enables the feature (`enabled = true`).
4. User specifies paths for the trigger sound and the mapping of intensity levels to file paths.
5. User restarts the service.

## 4. Functional Requirements

### 4.1. Audio Playback Engine
1. The system MUST use the `rodio` library for audio playback.
2. Playback MUST be executed in a non-blocking manner within the async runtime (e.g., using `spawn_blocking`) to ensure it does not halt seismic data processing.
3. The system MUST support standard audio formats (MP3 is required; WAV/OGG supported by `rodio` default).
4. The system MUST output audio to the OS **default output device**. Device selection logic is out of scope.

### 4.2. Alert Logic Integration
1. **Trigger Phase**: Upon an STA/LTA trigger event, the system MUST play the configured "Trigger" sound file immediately.
2. **Reset Phase**: Upon an alert reset event, the system MUST play a sound file determined by the **maximum JMA Seismic Intensity** recorded during the alert.
3. **Intensity Mapping**: The system MUST support configuring distinct sound files for different JMA intensity classes (e.g., 1, 2, 3, 4, 5-, 5+, 6-, 6+, 7).
4. **Fallback & Logging**: If a specific sound file for the observed JMA intensity is not configured, the system MUST:
    - Play a designated **default reset sound**.
    - Output an **error log** indicating the missing configuration for that intensity level.
5. **Preemption**: A new audio playback request (e.g., a new Trigger) MUST immediately stop any currently playing audio before starting the new file.

### 4.3. Configuration
1. The feature MUST be configurable via `rsudp.toml` in the `[ALERTSOUND]` section.
2. **Enabled/Disabled**: A boolean flag (`enabled`) to toggle the entire feature.
3. **Trigger Sound**: A file path for the initial alert sound.
4. **Intensity Sounds**: A mapping of JMA intensity classes to file paths.
5. **Default Reset Sound**: A file path for the fallback sound when a specific intensity sound is missing.

## 5. Success Criteria

1. **Latency**: Audio playback starts within 500ms of the internal trigger/reset event.
2. **Stability**: Audio playback does not cause the main data processing loop to block or drop packets.
3. **Accuracy**: The Reset sound correctly matches the calculated maximum intensity of the event.
4. **Resilience**: If a sound file is missing or corrupt, the system logs an error but continues operation (does not crash).

## 6. Assumptions & Constraints

*   **Hardware**: The server (e.g., Raspberry Pi) has a functional audio output device (speakers/jack).
*   **OS Configuration**: The ALSA/sound system is configured such that the desired output is set as the system default. The application will not manage device routing.
*   **Environment**: The application is running directly on Linux (systemd service or binary), NOT inside a Docker container (as per user instruction).
*   **Concurrency**: Audio playback is "fire and forget" or sequential; strict mixing of multiple simultaneous alert sounds is not a primary requirement, but the Trigger sound usually stops or finishes before Reset.

## 7. Security Considerations

*   **File Access**: The application reads audio files from the local filesystem. Paths should be validated to prevent directory traversal if configured via an external API (though currently config is file-based).