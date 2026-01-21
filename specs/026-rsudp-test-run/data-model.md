# Data Model: Test Run Configuration

## Entities

### `ComparisonConfig` (Implicit)
| Field | Value | Description |
|-------|-------|-------------|
| Input File | `references/mseed/fdsnws.mseed` | Source seismic data |
| Python Log | `logs/rsudp_python.log` | Output from reference run |
| Rust Log | `logs/rsudp_rust.log` | Output from target run |
| Report | `logs/comparison_report.csv` | Final analysis |

### `Settings Override`
User provided JSON to be converted to:
- `references/rsudp/rsudp/settings.json` (Python)
- `rsudp-rust/settings.toml` (Rust)

## Log Format
Expected log lines to parse:
- **Trigger**: `Trigger threshold <THRESH> exceeded at <TIMESTAMP>`
- **Reset**: `Earthquake trigger reset ... at <TIMESTAMP>`
