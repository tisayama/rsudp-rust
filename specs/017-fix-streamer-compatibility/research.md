# Research: Streamer Compatibility with rsudp

## Decisions

### Decision: Custom String Formatting instead of JSON
**Rationale**: `rsudp` uses a naive string parsing approach (`replace('}', '').split(',')`) rather than a standard JSON parser. Using `serde_json` produces valid JSON (`[...]`), but `rsudp` fails to parse the trailing `]` because it expects a specific string structure ending with `}`. We must manually construct the string to match `rsudp`'s expectation: `"{ 'CHANNEL', TIMESTAMP, SAMPLE1, SAMPLE2, ... }"` (or similar, based on `getSTREAM` implementation).
**Alternatives considered**:
- **Patch rsudp**: Not an option as we aim for compatibility with the existing installed base.
- **Custom Serde Serializer**: Possible, but overkill for a single packet format. Manual `format!` macro is simpler and more explicit about the exact byte structure required.

### Decision: Timestamp Precision
**Rationale**: `rsudp` expects a float for timestamp (seconds since epoch). Rust's `chrono` timestamp should be formatted as a float (e.g., `1234567890.123456`) to ensure `rsudp`'s `float()` conversion succeeds.

## Research Tasks

### Task: Confirm Exact String Format
**Finding**: Based on `references/rsudp/rsudp/raspberryshake.py`:
- `getCHN`: `DP.decode('utf-8').split(",")[0][1:]` -> Expects first element to start with a char (like `{`) then the channel name. `strip("'")` is called on the result. So `{'CHANNEL'` works.
- `getTIME`: `float(DP.split(b",")[1])` -> Second element is timestamp.
- `getSTREAM`: `list(map(int, DP.decode('utf-8').replace('}','').split(',')[2:]))` -> Removes `}`, splits by comma, takes elements from index 2 onwards as integers.
**Conclusion**: The format must be exactly: `{'CHANNEL', TIMESTAMP, SAMPLE1, SAMPLE2, ..., SAMPLEN}`.
- Start with `{`
- Channel in single quotes
- Timestamp as float
- Samples as integers
- End with `}`
- Separated by `,` (no space requirement strictly, but `split(',')` implies it).

### Task: Handling Negative Integers
**Finding**: `rsudp` maps `int()` over the split strings. Python's `int()` handles negative strings ("-123") correctly.
