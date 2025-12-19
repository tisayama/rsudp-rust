# Research: Japanese Seismic Intensity Calculation

## Decision: Calculation Method (Frequency Domain vs. Time Domain)

- **Decision**: Implement JMA standard calculation using Frequency Domain filtering (FFT/IFFT).
- **Rationale**: The official JMA specification defines the filters (Period Effect, High-Cut, Low-Cut) in the frequency domain. While time-domain IIR/FIR approximations exist, using `rustfft` to apply the exact JMA filter response ensures maximum compliance and accuracy (SC-003). For real-time monitoring, a 60-second sliding window with 1-second updates is computationally feasible.
- **Alternatives Considered**: Time-domain IIR filters (Direct Form I/II). Rejected for this phase to prioritize accuracy and ease of implementation of the complex JMA filter response.

## Decision: Sliding Window Strategy

- **Decision**: Use a 60-second buffer (6000 samples at 100Hz) for each of the 3 channels.
- **Rationale**: JMA intensity calculation requires a sufficient data duration for stable frequency analysis. A 60-second window is the standard practice for instrumental intensity meters.
- **Update Frequency**: Recalculate every 1 second to provide a "current" intensity during an ongoing event.

## Decision: Unit Conversion (Counts to Gal)

- **Decision**: Use a simple linear sensitivity factor: `Gal = Counts / Sensitivity`.
- **Rationale**: Requirement FR-008 and Clarification Session 2025-12-19 specify internal conversion. Most seismic sensors provide a sensitivity value (counts per m/s² or counts per gal).

## Decision: Determination of 'a' (0.3s Cumulative Duration)

- **Decision**: Sort the vector-sum acceleration values in descending order and pick the value at the 0.3-second mark (e.g., the 30th largest value in a 100Hz stream).
- **Rationale**: This is the standard algorithm for determining the acceleration threshold `a` that is exceeded for a cumulative duration of 0.3 seconds.
