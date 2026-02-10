# Quickstart: Alert Audio Playback

## Setup

1.  **Install Dependencies**:
    Ensure ALSA development headers are installed on your Linux system (required for `rodio` build).
    ```bash
    sudo apt-get install libasound2-dev
    ```

2.  **Prepare Audio Files**:
    Place your MP3 or WAV files in a directory (e.g., `./sounds/`).
    - `trigger.mp3`
    - `reset_level_1.mp3`
    - `reset_level_4.mp3`
    - etc.

3.  **Configure `rsudp.toml`**:
    Edit your configuration to enable sound and map the files.

    ```toml
    [ALERTSOUND]
    enabled = true
    trigger_file = "./sounds/trigger.mp3"
    default_reset_file = "./sounds/reset_default.mp3"
    
    [ALERTSOUND.intensity_files]
    "1" = "./sounds/reset_level_1.mp3"
    "4" = "./sounds/reset_level_4.mp3"
    "5+" = "./sounds/reset_level_5p.mp3"
    ```

4.  **Run**:
    Start the application normally.
    ```bash
    ./target/release/rsudp-rust
    ```

## Verification

1.  **Trigger Test**:
    Send a test UDP packet or replay a data file that triggers an alert.
    - **Expectation**: The `trigger.mp3` plays immediately.

2.  **Preemption Test**:
    While a long `reset` sound is playing, trigger a new alert.
    - **Expectation**: The `reset` sound stops instantly, and `trigger.mp3` starts.

3.  **Intensity Mapping Test**:
    Trigger an alert that results in Intensity 4.
    - **Expectation**: After the alert resets, `reset_level_4.mp3` plays.
