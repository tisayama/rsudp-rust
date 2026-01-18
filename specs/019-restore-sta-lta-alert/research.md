# Research: Trigger Logic Regression

## Decisions

### Decision: Validate Filter State Persistence
**Rationale**: The most likely cause of failure with small chunks is that the `BandpassFilter` or STA/LTA state is resetting or not accumulating correctly between calls. If `add_sample` is called per-packet, the state must persist perfectly.
**Hypothesis**: The filter initialization logic (`!self.initialized`) might be re-triggering, or `x1`/`x2` states are lost if the `StaLtaState` is dropped/recreated (unlikely if `HashMap` persists, but worth checking). Or, the `BandpassFilter` implementation assumes a continuous block of data and doesn't handle single-sample updates correctly in its IIR recursion.

### Decision: Check IIR Filter Implementation
**Rationale**: `Biquad` filter logic relies on `x1`, `x2` (previous inputs) and `y1`, `y2` (previous outputs). If these are reset, the filter "rings" or zeros out.
**Action**: Verify `BandpassFilter::process` correctly updates and holds state in `self.sections`.

## Research Tasks

### Task: Unit Test with Chunked Data
**Decision**: Create a unit test in `trigger.rs` or `integration_alert.rs` that feeds data in 25-sample chunks vs 1000-sample chunks and asserts the result is identical.
