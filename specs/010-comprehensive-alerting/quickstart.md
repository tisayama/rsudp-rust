# Quickstart: Comprehensive Alerting System

## Overview
This feature adds real-time visual/audio alerts, automatic waveform snapshots, and email notifications to `rsudp-rust`.

## Setup

### 1. SMTP Configuration
Configure your SMTP server in `Settings` via the WebUI or environment variables (to be implemented):
- `SMTP_HOST`: e.g., `smtp.gmail.com`
- `SMTP_PORT`: `587`
- `SMTP_USER`: your email
- `SMTP_PASS`: your app password
- `ALERT_RECIPIENT`: where notifications are sent

### 2. Assets
Ensure alert sounds are placed in `webui/public/sounds/alert.wav`.

## Verification Steps

### 1. Visual & Audio Alert
1. Start `rsudp-rust` and open the WebUI.
2. Use the `streamer` to play a MiniSEED file containing a significant seismic event (e.g., `fdsnws.mseed`).
3. **Expectation**: The WebUI background should flash red, and the alert sound should play within 500ms of the trigger.

### 2. Snapshot Generation
1. Wait for the seismic event to conclude (ratio drops below reset threshold).
2. Check the `rsudp-rust/alerts/` directory.
3. **Expectation**: A PNG file should be generated showing the waveform before and after the trigger.

### 3. Email Notification
1. Check the `ALERT_RECIPIENT` inbox.
2. **Expectation**:
   - One "Event Triggered" email received immediately.
   - One "Event Summary" email received after reset, containing stats and a link to the snapshot.

### 4. Alert History
1. Navigate to `/history` in the WebUI.
2. **Expectation**: The event should be listed with its peak STA/LTA ratio and a thumbnail of the snapshot.
