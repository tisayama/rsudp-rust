# Data Model: Streaming STA/LTA Trigger Calculation

**Feature**: 042-streaming-sta-lta | **Date**: 2026-03-02

## Entity: StaLtaState (MODIFIED)

Per-channel streaming state. Replaces the current buffer-based state.

### Current Fields (to be removed)
| Field | Type | Description |
|-------|------|-------------|
| ~~buffer~~ | ~~VecDeque\<f64\>~~ | ~~Sample buffer (3100 elements) — REMOVED~~ |

### New/Modified Fields
| Field | Type | Initial Value | Description |
|-------|------|---------------|-------------|
| filters | Vec\<Biquad\> | `butter_bandpass_sos()` output | Persistent bandpass filter sections (4 Biquads with s1/s2 state) |
| sta | f64 | 0.0 | Current short-term average (exponentially-weighted) |
| lta | f64 | 1e-99 | Current long-term average (exponentially-weighted, 1e-99 for zero-division prevention) |
| sample_count | usize | 0 | Number of samples processed since last reset (for warmup tracking) |

### Preserved Fields (unchanged)
| Field | Type | Description |
|-------|------|-------------|
| triggered | bool | Whether currently in triggered (alarm active) state |
| max_ratio | f64 | Maximum STA/LTA ratio since last trigger |
| last_timestamp | Option\<DateTime\<Utc\>\> | Timestamp of last processed sample (for gap detection) |
| exceed_start | Option\<DateTime\<Utc\>\> | Start time of current threshold exceedance (for duration-based triggering) |
| is_exceeding | bool | Whether currently exceeding threshold (for duration-based triggering) |

### State Transitions

```
[Init] ──(first sample)──► [Warming Up]
                              │
                    (sample_count >= nlta)
                              │
                              ▼
                         [Ready] ◄──(gap > 1s)──► [Init]
                           │   ▲
              (ratio > threshold) (ratio < reset)
                           ▼   │
                        [Triggered]
```

## Entity: TriggerManager (UNCHANGED)

| Field | Type | Description |
|-------|------|-------------|
| config | TriggerConfig | Trigger parameters (STA/LTA seconds, thresholds, filter freqs, target channel, duration) |
| states | HashMap\<String, StaLtaState\> | Per-channel state instances |

## Entity: Biquad (UNCHANGED structure, CHANGED usage)

| Field | Type | Description |
|-------|------|-------------|
| b0, b1, b2 | f64 | Numerator coefficients (constant after initialization) |
| a1, a2 | f64 | Denominator coefficients (constant after initialization) |
| s1, s2 | f64 | Filter state variables (**NOW persistent across samples**) |

### Usage Change
- **Before**: Created fresh with `s1=0.0, s2=0.0` on every `add_sample()` call
- **After**: Created once per channel, `s1/s2` updated by `process()` and persist; only reset on data gap

## Entity: AlertEvent (UNCHANGED)

No changes to the event structure or types (Trigger, Reset, Status).

## Entity: TriggerConfig (UNCHANGED)

No changes to configuration parameters.

## Processing Flow (per sample)

```
1. Receive sample (value, timestamp, channel_id)
2. Check channel match (target_channel filter)
3. Get/create per-channel StaLtaState
4. Gap detection:
   - If (timestamp - last_timestamp) > 1s → reset filters/STA/LTA/counter
5. Update last_timestamp
6. Apply bandpass filter: val = filter_chain.process(sample)
7. Compute energy: energy = val * val
8. Update STA: sta = (1/nsta) * energy + (1 - 1/nsta) * sta
9. Update LTA: lta = (1/nlta) * energy + (1 - 1/nlta) * lta
10. Increment sample_count
11. If sample_count < nlta → ratio = 0.0 (warmup)
12. Else → ratio = sta / lta
13. Apply trigger logic (threshold/reset/duration/status)
14. Return Option<AlertEvent>
```
