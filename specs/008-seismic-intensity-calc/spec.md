# Feature Specification: Japanese Seismic Intensity Calculation

**Feature Branch**: `008-seismic-intensity-calc`  
**Created**: 2025-12-19  
**Status**: Draft  
**Input**: User description: "以下のC言語実装を参考に、日本規格の計測震度(seismic intensity)を計算し、震度階級を判定できる処理を作ってください。 https://github.com/ingen084/seismometer ... あらかじめ利用したい3つのチャンネルを設定し、そのチャンネルのデータから計算するようにしてください。 実装ができたら ... MiniSEEDデータで震度階級の判定を行ってください。 入力データ - `references/mseed/fdsnws.mseed` ... ENE, ENN, ENZチャンネルを利用して、計測震度と震度階級を算出してください。 期待結果 震度階級: 2か3程度"

## Summary

Implement the calculation of Japanese Measured Seismic Intensity (計測震度) and the determination of JMA Seismic Intensity Classes (震度階級) based on three-component acceleration data. The implementation will follow the official Japan Meteorological Agency (JMA) standard, utilizing digital filters (period effect, high-cut, and low-cut) and vector sum analysis of 3-axis acceleration. This feature allows users to configure three specific input channels (e.g., ENE, ENN, ENZ) and provides real-time or batch calculation of seismic intensity from MiniSEED or internal data streams.

## Clarifications

### Session 2025-12-19
- Q: 計算の実行タイミングと更新頻度は？ → A: **Continuous update** (常に最新の60秒間を対象に、1秒ごとに計算を更新する)
- Q: 入力データの単位（Counts vs 物理単位）の扱いは？ → A: **Internal conversion** (Countsと感度情報を受け取り、内部でGalに変換する)
- Q: 計算結果の出力方法は？ → A: **WebUI Integration** (WebSocket経由でWebUIにリアルタイム配信・表示する)

## User Scenarios & Testing *(mandatory)*
...
### Functional Requirements

- **FR-001**: The system MUST allow configuration of three specific data channels to be used for the 3-axis calculation (e.g., ENE, ENN, ENZ).
- **FR-002**: The system MUST implement the JMA digital filtering process, including period effect weighting, high-cut filter, and low-cut filter.
- **FR-003**: The system MUST calculate the vector sum (root-sum-square) of the three filtered components.
- **FR-004**: The system MUST determine the acceleration value `a` that is exceeded for a cumulative duration of 0.3 seconds within a sliding 60-second window, updated every 1 second.
- **FR-005**: The system MUST calculate the instrumental intensity `I` using the formula `I = 2 log10(a) + 0.94`.
- **FR-006**: The system MUST map the value `I` to the 10 JMA intensity classes (0-7, with 5 and 6 split into Lower/Upper).
- **FR-007**: The system MUST support processing MiniSEED files as input for verification (e.g., `references/mseed/fdsnws.mseed`).
- **FR-008**: The system MUST broadcast the calculated intensity and class to the WebUI via WebSockets in real-time.

### Key Entities *(include if feature involves data)*

- **IntensityConfig**: Configuration containing the 3 target channel IDs and sampling rate.
- **AccelerationBuffer**: Storage for 3-component acceleration samples used for filtering and peak detection.
- **IntensityResult**: Object containing the raw instrumental intensity value and the corresponding JMA class name.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Accuracy: The calculated intensity for `references/mseed/fdsnws.mseed` using ENE, ENN, ENZ channels must result in a JMA class of 2 or 3.
- **SC-002**: Performance: Real-time calculation keeps pace with 100Hz data with < 100ms latency.
- **SC-003**: Compliance: The digital filter response matches the JMA standard within a 1% error margin across the 0.1Hz - 10Hz range.

## Edge Cases

- **Missing Channel**: How the system handles cases where only 1 or 2 of the 3 configured channels are providing data (should log error and wait for all components).
- **Sampling Rate Mismatch**: If the 3 channels have different sampling rates (should error out as JMA intensity requires synchronized 3-axis data).
- **Short Data Duration**: Calculating intensity when less than 60 seconds of data is available (should provide "estimated" intensity or wait for buffer to fill).

## Assumptions



- The system handles the conversion from digital counts to acceleration (Gal) internally using provided sensitivity metadata.

- The 3 configured channels are orthogonal (X, Y, Z components).

- Time synchronization across the 3 channels is maintained by the data ingestion pipeline.
