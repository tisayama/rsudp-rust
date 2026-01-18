# Data Model: UDP Packet Format

## Entities

### `UDPPacket` (Wire Format)
The payload sent over UDP to `rsudp`. This is NOT a standard JSON object but a Python-string-representation-like structure.

**Format Pattern**:
`{'CHANNEL', TIMESTAMP, SAMPLE_1, SAMPLE_2, ..., SAMPLE_N}`

**Fields**:

| Field | Type | Format Detail | Example |
|-------|------|---------------|---------|
| Start Marker | char | `{` | `{` |
| Channel | string | Single-quoted string | `'EHZ'` |
| Timestamp | float | Seconds since Epoch (Unix) | `1678886400.123` |
| Samples | int[] | Comma-separated integers | `100, 102, 99` |
| End Marker | char | `}` | `}` |

**Example Payload**:
`{'EHZ', 1678886400.123, 100, 102, 99, ...}`

**Validation Rules**:
1. Must start with `{`.
2. Must end with `}`.
3. Channel must be the first element after `{`.
4. Timestamp must be the second element.
5. All subsequent elements must be valid integers parsable by Python's `int()`.
6. Separator must be `,` (comma). Space after comma is optional but standard in Python stringification.

## State Transitions
N/A - This is a stateless packet format definition.
