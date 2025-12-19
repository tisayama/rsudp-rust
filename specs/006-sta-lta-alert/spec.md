# Specification: STA/LTA Alert System

**Status**: Draft
**Feature Number**: 006
**Short Name**: sta-lta-alert
**Owner**: [ASSIGNED_OWNER]

## Summary

Implement an alert system that monitors seismic data streams in real-time, calculates the Short-Term Average over Long-Term Average (STA/LTA) ratio, and triggers notifications when seismic events are detected. This feature aims to provide functionality similar to the alert system in `rsudp`, enabling automated event detection based on signal energy changes.

## Clarifications

### Session 2025-12-19
- Q: 1つのインスタンスで複数のチャンネルを同時に監視するか、それともチャンネルごとに個別のインスタンスを動作させるか？ → A: **Single-channel** (各インスタンスは指定された1つのチャンネルのみを監視する)
- Q: データ欠損（ギャップ）検出時の挙動は？ → A: **Reset** (状態をリセットし、LTA期間分の再ウォームアップを行う)
- Q: アラートイベントに含まれるメタデータは？ → A: **Standard (rsudp-like)** (解除イベントに、その期間中の「最大STA/LTA比」を含める)

## User Scenarios

### Scenario 1: Seismic Event Detection
A seismic station operator wants to be automatically notified when an earthquake or significant vibration occurs. They configure the STA/LTA alert system with specific window lengths and a threshold. When a seismic wave arrives, the signal energy increases significantly, causing the STA/LTA ratio to exceed the threshold. The system immediately emits an alarm event.

### Scenario 2: Automatic System Reset
After a seismic event has passed and the ground motion subsides, the operator wants the system to return to a monitoring state automatically. As the signal energy decreases, the STA/LTA ratio falls below a predefined "reset" threshold. The system detects this, emits a reset event, and resumes monitoring for the next event.

### Scenario 3: Noise Reduction via Filtering
A user is monitoring a station in a noisy environment (e.g., near a road). They want to ignore low-frequency traffic noise and focus on higher-frequency seismic signals. They configure a bandpass filter (e.g., 1-20 Hz) to be applied to the data stream before the STA/LTA calculation, reducing false triggers from non-seismic sources.

## Functional Requirements

1.  **Real-time STA/LTA Calculation**:
    - Implement a recursive STA/LTA algorithm that processes incoming data samples.
    - Support configurable window lengths for Short-Term Average (STA) and Long-Term Average (LTA) in seconds.
2.  **Trigger Logic**:
    - Trigger an "ALARM" state when the calculated STA/LTA ratio exceeds a configurable `threshold`.
    - Provide an optional `min_duration` parameter: the threshold must be exceeded for this amount of time before an alarm is officially triggered.
3.  **Reset Logic**:
    - Reset the system to "MONITORING" state when the STA/LTA ratio falls below a configurable `reset_threshold`.
4.  **Preprocessing (Filtering)**:
    - Support optional digital filters (Bandpass, Highpass, Lowpass) to be applied to the data stream before calculating averages.
5.  **Event Notification**:
    - Emit an "ALARM" event when triggered, containing the trigger timestamp and the channel ID.
    - Emit a "RESET" event when the system returns to monitoring, containing the reset timestamp, the channel ID, and the **maximum STA/LTA ratio** recorded during the alarm duration.
6.  **Warm-up Period**:
    - The system must handle an initial "warm-up" period equal to the LTA window length before alerts can be reliably triggered.
7.  **Channel Selection**:
    - Each instance monitors exactly one specified data channel (e.g., "SHZ", "EHZ"). Multiple channels are handled by deploying multiple instances.

## Success Criteria

1.  **Accuracy**: The system triggers alarms correctly when tested against seismic data with known events, matching the behavior of standard STA/LTA implementations (like ObsPy).
2.  **Latency**: Alarm events are emitted within 200ms of the threshold being exceeded in the processed data stream.
3.  **Stability**: The calculation keeps pace with a 100Hz data stream on standard hardware with minimal CPU usage (< 5% of a single core).
4.  **Verifiability**: Both Alarm and Reset events are captured in logs with accurate UTC timestamps.

## Key Entities

- **AlertConfig**: Stores parameters like STA/LTA lengths, thresholds, and filter settings.
- **AlertState**: Tracks the current state (Warming Up, Monitoring, Triggered).
- **SeismicSample**: The input data unit.
- **AlertEvent**: The output notification (Alarm or Reset).

## Edge Cases & Error Handling

- **Data Gaps**: If a gap is detected in the incoming data stream (discontinuity in timestamps), the STA/LTA calculation state must be reset. The system enters a "Warming Up" state for a duration equal to the LTA window before resuming monitoring.
- **Out-of-Order Data**: Samples arriving with timestamps earlier than the last processed sample should be discarded to maintain the integrity of the recursive calculation.
- **Low Sample Rate**: If the sample rate is significantly lower than expected, the system should log a warning as it may affect the precision of the STA/LTA windows.

## Assumptions

- The input data stream provides a consistent sample rate.
- Time synchronisation is handled at the data ingestion level; the alert system uses the timestamps provided by the stream.
- The recursive STA/LTA algorithm is sufficient for real-time performance (as opposed to non-recursive variants).

## Dependencies

- Requires a data ingestion pipeline (Feature 004) to provide real-time samples.
- Requires basic STA/LTA calculation logic (Feature 003).

## Out of Scope

- Graphical visualization of the STA/LTA ratio (this would be a separate feature).
- Advanced deconvolution or instrument response correction (unless required for basic detection).
- Direct integration with external notification services (e.g., Telegram, Twitter) within this specific component; it should only emit internal events that other components can consume.