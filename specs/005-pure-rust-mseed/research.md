# Research: Pure Rust MiniSEED Ingestion

## Decision: Custom Implementation of MiniSEED 2 Parser

- **Decision**: Implement a custom parser for MiniSEED 2 Fixed Section Data Headers and a Steim1/2 decompression engine.
- **Rationale**: No high-quality, maintained Pure Rust crate exists that fully supports MiniSEED 2 with Steim2. By implementing the subset of the SEED 2.4 specification required for `rsudp`, we eliminate C dependencies and ensure long-term maintainability.
- **Alternatives Considered**: 
    - `mseedio`: Only targets MiniSEED 3 and lacks mature Steim decoding.
    - `msrepack`: Not available on crates.io and covers more than needed.

## MiniSEED 2 Header Reference

The parser will focus on the **Fixed Section Data Header (48 bytes)**:
- **NSLC Codes**: Station (5), Location (2), Channel (3), Network (2).
- **Start Time**: 10-byte BTIME structure (Year, Day, Hour, Min, Sec, Unused, 0.0001s).
- **Sample Rate**: Calculated via `factor` and `multiplier`.
- **Data Offset**: Indicates where the compressed frames begin.

## Steim2 Decompression Algorithm

Steim2 is a difference-based compression using 64-byte frames.
- **Integration Constants**: Word 1 (X0) and Word 2 (Xn) in the first frame.
- **Control Words**: Word 0 contains 16 2-bit flags (ck) determining the content of Words 1-15.
- **Difference Encoding**:
    - `ck=01`: 4 differences (8 bits each).
    - `ck=10`: Variable differences (determined by leading 2 bits: `01`=1x30bit, `10`=2x15bit, `11`=3x10bit).
    - `ck=11`: Variable differences (determined by leading 2 bits: `00`=5x6bit, `01`=6x5bit, `10`=7x4bit).
- **Reconstruction**: `X[i] = X[i-1] + diff[i]`. Verify `X[last] == Xn`.

## Dependency Selection

- `byteorder`: Essential for Big-Endian parsing of SEED records.
- `chrono`: For representation of seismic event timestamps.
