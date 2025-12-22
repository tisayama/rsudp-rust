# Research: rsudp-style Alert Post Timing

## Decisions & Rationale

### 1. Task Scheduling Mechanism
- **Decision**: Use `tokio::spawn` with `tokio::time::sleep` for scheduling the delayed snapshot generation.
- **Rationale**: This is the most idiomatic way in a Tokio-based async application to perform a delayed action without blocking the primary data processing loop. It matches `rsudp`'s behavior of "scheduling" a save event.
- **Alternatives considered**: 
    - Manual timer check in the `pipeline` loop: Rejected because it adds complexity to the loop and requires managing a collection of pending tasks.
    - `tokio::time::Interval`: Not suitable for one-off delayed tasks.

### 2. Time Reference (Real-time vs Data-time)
- **Decision**: Use **Real-time (Wall-clock)** for the delay calculation, consistent with `rsudp`'s use of `time.time()`.
- **Rationale**: The user specifically requested to mimic `rsudp`. In `rsudp`, the `save_at` is calculated as `now + delay`. While this can be slightly inaccurate during high-speed simulations, it provides the expected behavior for live monitoring.
- **Note**: For extremely high-speed simulations where 60 seconds of data passes in 1 second of wall-clock time, a real-time timer might trigger after the data has already passed the buffer. However, given the 300s buffer implemented in Feature 011, this is unlikely to be an issue for standard test speeds.

### 3. Intensity Calculation Window
- **Decision**: The "max intensity" included in the message will be the peak value recorded between the `Trigger` time and the `Post` time (Scheduled Execution time).
- **Rationale**: Matches the requirement "その時点までの最大震度階級を含む詳細メッセージ".

## Best Practices Found

### Async State Management
- When spawning a background task for image generation, the task needs access to the `WebState` and the `waveform_buffers`. 
- Since `waveform_buffers` are local to the pipeline loop, they must be cloned or a snapshot taken at the time of execution. 
- To ensure the snapshot contains the correct window, the background task should receive a clone of the necessary buffer slice or the `pipeline` should wait until the time is right to extract the data.

### Implementation Strategy
- Upon `Trigger`:
    1. Calculate `delay = duration * save_pct`.
    2. Spawn a task: `sleep(delay).await` -> `Generate Snapshot & Send Notification`.
- This decouples the `RESET` event from the notification logic.
