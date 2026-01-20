# Research: Fix Trigger Parameters

## Decisions

### Decision: Filter Coefficient Calculation (IIR Butterworth)
**Rationale**: `scipy.signal.butter` uses bilinear transform to design digital filters. To match `rsudp` behavior, we must implement the exact same math in Rust.
**Choice**: Implement a standalone `butter` function in `src/filter.rs` (or similar) that takes order, low/high frequencies, and sample rate, and outputs SOS (Second-Order Sections) coefficients.
**Algorithm**:
1. Pre-warp frequencies: `omega = 2 * fs * tan(pi * freq / fs)`
2. Design analog prototype (poles on unit circle in s-plane).
3. Bilinear transform to z-plane.
4. Group poles/zeros into biquad sections.

### Decision: Parameter Logging
**Rationale**: Silent defaults are causing confusion.
**Choice**: In `TriggerManager::new`, explicitly log all effective parameters (calculated coefficients, sample counts for STA/LTA windows) at `INFO` level.

### Decision: Dynamic Configuration
**Rationale**: The current implementation uses hardcoded filter coefficients.
**Choice**: `TriggerManager` must accept `highpass` and `lowpass` values and call the new `butter` function during initialization (and whenever config changes, if supported).

## Research Tasks

- [x] Identify `scipy.signal.butter` algorithm (Bilinear transform of analog prototype).
- [x] Confirm `rsudp` defaults (4th order, bandpass).
- [ ] Implement `butter` function in Rust (Tasks phase).
