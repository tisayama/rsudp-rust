# Tasks: Fix Trigger Parameters for Parity

**Feature**: Fix Trigger Parameters for Parity
**Status**: Completed
**Implementation Strategy**: Implement dynamic IIR filter coefficient calculation using the bilinear transform to match `scipy.signal.butter`. Verify parameter mapping for STA/LTA windows and log all effective settings at startup.

## Phase 1: Setup
Goal: Prepare for dynamic filter calculation.

- [X] T001 Verify `num-complex` dependency is available in `rsudp-rust/Cargo.toml`

## Phase 2: Foundational
Goal: Implement filter design logic.

- [X] T002 Implement `butter_bandpass_sos` function in `rsudp-rust/src/trigger.rs` to calculate 4th-order Butterworth coefficients dynamically
- [X] T003 Update `BandpassFilter::new` in `rsudp-rust/src/trigger.rs` to accept `low_freq`, `high_freq`, and `sample_rate`, and use the new calculation logic

## Phase 3: [US1] Parameter Integration (Priority: P1)
Goal: Ensure config values are correctly applied.

- [X] T004 [US1] Update `TriggerManager::new` in `rsudp-rust/src/trigger.rs` to initialize `BandpassFilter` with `config.highpass` and `config.lowpass`
- [X] T005 [US1] Add INFO logging in `TriggerManager::new` to output effective STA/LTA sample counts and filter band
- [X] T006 [US1] Ensure `sta_sec` and `lta_sec` are converted to samples using the correct sampling rate (100Hz default)

## Phase 4: Polish & Cross-Cutting Concerns
Goal: Final verification.

- [X] T007 Verify parameter application by running `rsudp-rust` and checking startup logs
- [ ] T008 Perform independent test with `streamer` and user-provided config to confirm parity with `rsudp`

## Dependencies

- Phase 3 depends on Phase 2 filter logic.
- T007/T008 depend on T005.

## Parallel Execution Examples

- T002 (Filter Math) can be implemented independently of T004 (Integration).

## Implementation Strategy

1. **Math**: Implement the core `butter` algorithm.
2. **Integration**: Wire it into the `TriggerManager`.
3. **Verification**: Confirm logs and runtime behavior match expectations.