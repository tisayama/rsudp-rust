# Quickstart: rsudp-rust Configuration

## Installation

Add the following to your `rsudp_settings.toml` in `~/.rsudp/`:

```toml
[settings]
station = "AM.R6E01"
port = 12345

[alert]
threshold = 4.5
sta = 5.0
lta = 40.0

[plot]
enabled = true
duration = 90
```

## Running the Application

Start with the default config:
```bash
rsudp-rust
```

Override with a specific config file:
```bash
rsudp-rust --config ./my_settings.yaml
```

Override via Environment Variables:
```bash
RUSTRSUDP_SETTINGS__PORT=9999 rsudp-rust
```

Override via CLI arguments (if supported by specific flags):
```bash
rsudp-rust --station TEST1
```

## Dumping Default Config

To create a template:
```bash
rsudp-rust --dump-config settings.toml
```
