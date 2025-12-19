# Research: STA/LTA Algorithm

## Decision: Recursive STA/LTA Variant

- **Decision**: Implement the recursive STA/LTA algorithm matching `obspy.signal.trigger.recursive_sta_lta`.
- **Rationale**: This variant is computationally efficient (O(1) per sample) and suitable for real-time processing, which is a requirement for `rsudp`.
- **Formula**:
    ```
    csta = 1.0 / nsta
    clta = 1.0 / nlta
    sta_new = sta_old * (1.0 - csta) + (input * input) * csta
    lta_new = lta_old * (1.0 - clta) + (input * input) * clta
    ratio = sta_new / lta_new
    ```
    Note: The classic recursive algorithm typically operates on the square of the amplitude (energy).

## Decision: Testing Strategy

- **Decision**: Use a Python script to generate a reference CSV file containing input data and expected STA, LTA, and ratio values. The Rust test will read this CSV and compare its calculations.
- **Rationale**: Directly calling Python from Rust during tests can be fragile (environment issues). Generating a static or dynamic reference file is more robust and portable. `obspy` is the ground truth.
