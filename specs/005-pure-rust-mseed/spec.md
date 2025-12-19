# Feature Specification: Pure Rust MiniSEED Ingestion

**Feature Branch**: `005-pure-rust-mseed`
**Created**: 2025-12-19
**Status**: Draft
**Input**: User description: "MiniSEEDのパースにC言語のライブラリを使っていますがRustネイティブにできませんか？"

## Clarifications

### Session 2025-12-19

- Q: Pure Rust での移行にあたり、既存の Pure Rust ライブラリを利用しますか、それとも自前で実装しますか？ → A: 既存の Pure Rust ライブラリを利用する。
- Q: Steim1/2 以外のエンコーディング（ASCII, 16/32bit整数など）への対応は必須ですか？ → A: 全ての主要形式（非圧縮整数を含む）をサポートする。
- Q: 全形式をサポートする Pure Rust ライブラリがない場合の優先順位は？ → A: Pure Rust を優先する（ライブラリ不足分は自前実装を検討）。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build without C Toolchain (Priority: P1)

As a developer on a restricted environment, I want to build and run the project using only the Rust toolchain (without `clang` or `libmseed`), so that the development setup is simple and portable.

**Why this priority**: Removing the C dependency simplifies CI/CD and onboarding for new developers.

**Independent Test**: Remove `clang` and the C-based `mseed` crate from the build environment, and verify that `cargo build` still succeeds.

**Acceptance Scenarios**:

1. **Given** a system with only Rust installed, **When** `cargo build` is run, **Then** the application compiles successfully without errors related to missing C headers or compilers.

---

### User Story 2 - Maintain Data Accuracy (Priority: P1)

As a data analyst, I want the Pure Rust MiniSEED parser to produce the exact same numerical results as the previous C-based implementation, so that I can trust the data for seismic analysis.

**Why this priority**: Correctness is non-negotiable for seismic monitoring.

**Independent Test**: Process the same MiniSEED sample file (`references/mseed/fdsnws.mseed`) using both implementations and verify that the extracted samples match 100%.

**Acceptance Scenarios**:

1. **Given** a valid MiniSEED file, **When** parsed by the Pure Rust implementation, **Then** the resulting `TraceSegment` metadata and sample data match the `004` implementation results.

---

### Edge Cases

- Handling of uncommon MiniSEED encodings (beyond Steim1/2).
- Corruption detection in the byte stream without `libmseed`'s mature validation.
- Large files exceeding memory if not streamed correctly.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST replace the C-based `mseed` crate with an existing Pure Rust crate (e.g., `msstream` or similar) to handle MiniSEED parsing without C dependencies.
- **FR-002**: The new parser MUST support MiniSEED 2 Data Records, including Fixed Section Data Headers.
- **FR-003**: The parser MUST support Steim1, Steim2, and other major encodings including uncompressed 16-bit and 32-bit integers.
- **FR-004**: The output of the parser MUST remain compatible with the `TraceSegment` structure defined in `004`.
- **FR-005**: The system MUST maintain the ability to process both UDP packets and files.

### Key Entities *(include if feature involves data)*

- **PureRustParser**: The new component responsible for decoding MiniSEED bytes without FFI.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo build` executes successfully on a clean system without `clang` or `build-essential`.
- **SC-002**: 100% match in sample values compared to the `004` implementation for the provided `fdsnws.mseed` file.
- **SC-003**: Parsing performance is within 20% of the C-based implementation (latency/throughput).