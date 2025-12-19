# Feature Specification: Data Ingestion Pipeline (UDP to STA/LTA)

**Feature Branch**: `004-data-ingestion-pipeline`
**Created**: 2025-12-19
**Status**: Draft
**Input**: User description: "UDPで受信したバイト列をパースして、STA/LTAの計算処理に渡す処理を実装してほしいです。MiniSEEDの複数ファイルのデータを入力してテストができるように途中の処理をMiniSEED準拠にできればそうしてほしいです。無理そうならMiniSEED準拠を挟まなくてよいです。"

## Clarifications

### Session 2025-12-19

- Q: MiniSEED形式のどの程度の準拠を目指しますか？ → A: フルスペック対応 (SEED/MiniSEEDのすべての制御情報を解釈する)
- Q: 複数のソースが届く場合、どのように識別してSTA/LTAフィルタを割り当てますか？ → A: 動的割り当て (NSLCキーによる自動生成)
- Q: MiniSEEDデータにおいて時間に隙間（ギャップ）がある場合、フィルタの状態をどうすべきですか？ → A: 一定時間以上のギャップでリセット (閾値超過でフィルタを初期化)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Real-time Data Pipeline (Priority: P1)

As a developer, I want the bytes received via UDP to be automatically parsed and fed into the STA/LTA calculation, so that the system can perform real-time seismic event detection.

**Why this priority**: This connects the ingestion and calculation components, forming the core functional path of the application.

**Independent Test**: Use `nc` to send MiniSEED-formatted (or agreed-upon simple format) bytes to the UDP port and verify that the STA/LTA ratio is updated and logged.

**Acceptance Scenarios**:

1.  **Given** the application is listening for UDP, **When** valid seismic data packets are received, **Then** the data is parsed into floating-point samples and passed to the `RecursiveStaLta` filter.
2.  **Given** a stream of data, **When** the parsing succeeds, **Then** the STA/LTA ratio is calculated for each sample and the resulting trigger status is available.

---

### User Story 2 - MiniSEED File Simulation (Priority: P2)

As a tester, I want to be able to feed multiple MiniSEED files into the processing pipeline instead of real UDP packets, so that I can verify the system's behavior against historical or synthetic datasets.

**Why this priority**: Crucial for verification, debugging, and ensuring the "same result as rsudp" principle for large datasets.

**Independent Test**: Run the application in "simulation mode" providing a list of MiniSEED files, and verify the output matches expectations.

---

### Edge Cases

- **Data Gaps**: If the time difference between the end of the previous record and the start of the current record for the same NSLC exceeds a configurable threshold (default: 10s), the associated `RecursiveStaLta` state MUST be reset.
- **Malformed MiniSEED**: Records that fail basic integrity checks (e.g., checksum, sequence number) should be logged and discarded.
- **Sampling Rate Mismatch**: If a record has a different sampling rate than the existing filter state, it should trigger a re-initialization of the filter.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST implement a parser that extracts numerical samples from incoming byte streams.
- **FR-002**: The system MUST support full MiniSEED parsing, including headers (sampling rate, time, channel) and data encoding (e.g., Steim).
- **FR-003**: The system MUST dynamically map incoming data to `RecursiveStaLta` instances based on the Network, Station, Location, and Channel (NSLC) codes found in the MiniSEED header.
- **FR-004**: The system MUST provide a CLI option to specify MiniSEED files as input for testing/simulation.
- **FR-005**: The pipeline MUST handle asynchronous data flow from the `Receiver` to the `Trigger`.

### Key Entities *(include if feature involves data)*

- **Sample**: A single numerical value with metadata (timestamp, channel).
- **Trace**: A continuous series of samples.
- **Parser**: Component responsible for conversion.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of valid MiniSEED packets are correctly converted to numerical samples without data loss.
- **SC-002**: The system can process MiniSEED files at a rate of at least 1,000,000 samples per second in simulation mode.
- **SC-003**: Integration tests prove that a known MiniSEED input produces the same STA/LTA output as the standalone algorithm test.