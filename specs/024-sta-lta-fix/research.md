# Research: Fix STA/LTA Trigger Behavior

## Decisions

### Decision: 4th Order Bandpass Filter
**Rationale**: `obspy.signal.filter.bandpass` defaults to 4 corners (cascaded 2nd order) if not specified. `rsudp` uses this default. The current Rust implementation likely defaults differently or implements a single biquad (2nd order).
**Choice**: Update `BandpassFilter` in `src/trigger.rs` to cascade 4 biquad sections (or 2 sections of 2nd order if using standard biquad terminology) to match Obspy's characteristics.

### Decision: Warm-up by Discarding Data
**Rationale**: `rsudp` calculates `wait_pkts` based on LTA duration and skips processing that many packets at startup. This prevents the initial "fill from zero" behavior that causes huge ratios.
**Choice**: Implement a `warmup_samples` counter in `TriggerManager` and return `None` until it exceeds `LTA * sample_rate`.

### Decision: Duration-based Debounce
**Rationale**: `rsudp` implements a timer that requires the threshold to be exceeded for `duration` seconds before triggering.
**Choice**: Add `triggered_duration` tracking to `StaLtaState`. Only transition to `Trigger` state if `ratio > threshold` for the required duration.

## Research Tasks

- [x] Verify `rsudp` filter order (Confirmed: default 4 corners).
- [x] Verify `rsudp` warmup logic (Confirmed: `wait_pkts` loop).
- [x] Verify `rsudp` trigger timer (Confirmed: `_is_trigger_with_timer`).
