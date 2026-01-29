# Feature Specification: Fix Max Intensity Spike in Noise Data

**Feature Branch**: `028-fix-intensity-spike`  
**Created**: 2026-01-29  
**Status**: Draft  
**Input**: User description: "地震が発生していない状態の実データを50分以上受信したところ、2分間だけMax Intensityがなぜか3.30まで急上昇しました。何か計算に問題がないか調べていただき、問題あれば修正していただきたいです。 過去のspecで具体的な計算式は検討いただいていますので、それも振り返りつつ進めてください。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reproduce and Eliminate Intensity Spike (Priority: P1)

As a monitoring user, I want the system to calculate seismic intensity accurately from real-world data so that I don't receive false alarms when no earthquake has occurred.

**Why this priority**: The reported issue (Max Intensity 3.30 during non-event) indicates a significant false positive bug that undermines trust in the system.

**Independent Test**: Can be fully tested by replaying the provided 60-minute dataset (`normal.mseed`) through the pipeline and verifying the output logs.

**Acceptance Scenarios**:

1. **Given** the 60-minute real-world seismic dataset (`normal.mseed`) containing only background noise, **When** the system processes this data stream, **Then** the logged `Max Intensity` should consistently remain at a low level (e.g., below 1.0 or consistent with observed noise floor) throughout the entire duration.
2. **Given** the specific timeframe around T00:52:00 (approx 3120 seconds in) where the spike was previously observed, **When** the fixed system processes this segment, **Then** no sudden jump in intensity value should be recorded.

### User Story 2 - Verify Calculation Logic Integrity (Priority: P2)

As a developer, I want to ensure the intensity calculation adheres to the previously defined specifications so that the fix doesn't introduce regressions or deviate from the standard JMA method.

**Why this priority**: Ensuring the fix is mathematically correct and not just a "patch" for one dataset.

**Independent Test**: Reviewing the calculation logic against the spec and potentially running synthetic test vectors.

**Acceptance Scenarios**:

1. **Given** the intensity calculation module, **When** checked against the logic defined in feature `008-seismic-intensity-calc` (and relevant updates), **Then** the implementation should correctly reflect the JMA intensity calculation steps (filtering, windowing, etc.).

### Edge Cases

- **DC Offset / Drift**: Real data often contains DC offsets or slow drifts. The calculation must handle these without producing artifacts.
- **Data Gaps / Discontinuities**: If the data contains packet losses or gaps, the filter should not "ring" or explode, causing a false spike.
- **Sudden Noise bursts**: Short duration non-seismic noise (bumps) should be handled gracefully.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST process continuous MiniSEED data streams without generating false intensity spikes > 2.0 when only background noise is present.
- **FR-002**: System MUST implement the JMA seismic intensity calculation algorithm as defined in previous specifications (referencing `008-seismic-intensity-calc`).
- **FR-003**: System MUST handle potential numeric instabilities (e.g., filter state initialization, very small/large numbers) that could lead to transient spikes.
- **FR-004**: System MUST log `Max Intensity` periodically (as per existing logic) to allow verification of the fix.

### Key Entities

- **IntensityManager**: The component responsible for processing waveform data and calculating the JMA intensity value.
- **RealTimeStream**: The 60-minute continuous data input used for validation (`normal.mseed`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: When processing the `normal.mseed` test dataset, the maximum recorded JMA Intensity is less than 1.5.
- **SC-002**: The specific window around T00:52:00 shows a stable intensity profile in the fixed implementation.
- **SC-003**: The calculation logic is verified to match the standard JMA algorithm steps without mathematical errors.
