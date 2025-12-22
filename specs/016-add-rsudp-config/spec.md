# Feature Specification: Implement rsudp-compatible Configuration System

**Feature Branch**: `016-add-rsudp-config`
**Created**: 2025-12-22
**Status**: Draft
**Input**: User description: "rsudpに倣った設定システムを追加したいです。同等の設定値を設定でき、設定ファイル形式はTOMLかYAMLのいずれかが利用できます。"

## User Scenarios & Testing *(mandatory)*

## Clarifications

### Session 2025-12-22
- Q: 設定値の優先順位 → A: コマンドライン引数 > 設定ファイル > デフォルト値
- Q: 機密情報の扱い → A: サポートする（環境変数が設定ファイルより優先される）
- Q: 未知のフィールドの扱い → A: 警告を表示して無視し、処理を続行する
- Q: デフォルトの設定ファイルパス → A: ユーザー의 홈 디렉토리 配下 (例: ~/.rsudp/settings.toml)
- Q: 複数の設定ファイル形式の同時存在 → A: TOML を優先する

### User Story 1 - Load Configuration from TOML File (Priority: P1)

The user wants to configure the application using a TOML file so that they can customize behavior (e.g., station name, port) using a familiar syntax.

**Why this priority**: TOML is a preferred configuration format in the Rust ecosystem and provides a clear, readable structure. This is the primary method for users to interact with the configuration system.

**Independent Test**: Create a valid `rsudp_settings.toml` file with non-default values (e.g., `port = 9999`). Run the application pointing to this file. Verify that the application starts and logs/displays the loaded configuration values correctly.

**Acceptance Scenarios**:

1. **Given** a `rsudp_settings.toml` file exists with valid settings, **When** the application is started with `--config rsudp_settings.toml`, **Then** the application successfully parses the file and applies the values (overriding defaults).
2. **Given** a `rsudp_settings.toml` file with missing fields, **When** the application is started, **Then** the application uses default values for the missing fields and loaded values for the present ones.
3. **Given** a malformed TOML file, **When** the application is started, **Then** it should exit with a clear error message indicating a parsing failure.

---

### User Story 2 - Load Configuration from YAML File (Priority: P2)

The user wants to configure the application using a YAML file, as they might be migrating from existing Python rsudp setups or prefer YAML.

**Why this priority**: Provides flexibility and compatibility with the original rsudp's preferred format (JSON/YAML-like structure) and user preference.

**Independent Test**: Create a `rsudp_settings.yaml` file with specific settings. Run the application pointing to this file. Verify values are loaded correctly.

**Acceptance Scenarios**:

1. **Given** a `rsudp_settings.yaml` file exists, **When** the application is started with `--config rsudp_settings.yaml`, **Then** the application successfully parses the file and applies the values.

---

### User Story 3 - Generate Default Configuration (Priority: P3)

The user wants to generate a default configuration file so that they have a template to start customizing.

**Why this priority**: Improves usability by helping users get started quickly without manually typing the entire schema.

**Independent Test**: Run the application with a `--dump-config` or similar flag. Verify a file is created with all default values populated.

**Acceptance Scenarios**:

1. **Given** no config file exists, **When** the user runs the command to dump config (e.g., `--dump-config rsudp_settings.toml`), **Then** a file is created containing all available settings with their default values in the specified format.

### Edge Cases

- **File Not Found**: If the specified config file does not exist, the application should probably error out (if explicitly provided) or warn and use defaults (if using a default path).
- **Unknown Fields**: If the config file contains keys not in the schema, the parser should likely ignore them or warn, but not crash.
- **Type Mismatch**: If a field expects an integer but gets a string, the parser should fail with a helpful error.
- **Unknown Fields**: The system MUST log a warning for any fields present in the configuration file that are not recognized by the schema, but continue processing using valid fields.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST define a configuration data structure that mirrors the `rsudp` settings schema.
    - Includes sections: `settings`, `printdata`, `write`, `plot`, `forward`, `alert`, `alertsound`, `custom`, `tweets`, `telegram`, `googlechat`, `discord`, `sns`, `line`, `bluesky`, `rsam`.
- **FR-002**: The system MUST provide default values for all settings, matching the defaults defined in `rsudp` (v1.0+).
- **FR-003**: The system MUST support parsing configuration files in **TOML** format.
- **FR-004**: The system MUST support parsing configuration files in **YAML** format.
- **FR-005**: The system MUST allow loading a configuration file via a command-line argument (e.g., `--config <PATH>`).
    - If no path is specified, the system SHOULD automatically search for and load a configuration file from a standard location (e.g., `~/.rsudp/settings.toml` or `~/.rsudp/settings.yaml`).
    - If both TOML and YAML files exist in the default location, TOML MUST take priority.
- **FR-006**: The system MUST correctly handle configuration priority and merging.
    - Priority (highest to lowest): Command-line arguments > Environment variables > Configuration file > Default values.
    - Environment variables are particularly supported for sensitive fields (e.g., API keys, tokens).
    - Defaults act as the base for any missing fields.
- **FR-007**: The system MUST support hierarchically nested settings (e.g., `plot.enabled`, `alert.threshold`).
- **FR-008**: The system SHOULD provide a mechanism to serialize the current (or default) configuration to a file (TOML/YAML) for user convenience.

### Key Entities

- **Settings**: The root configuration object containing all subsections.
- **SettingsSection**: General settings (port, station, output_dir, debug).
- **PlotSettings**: Plotting configuration (duration, refresh, spectrogram, filters).
- **AlertSettings**: Alert logic configuration (sta, lta, threshold, filtering).
- **ForwardSettings**: Data forwarding configuration.
- **NotificationSettings**: Generic structure for various notification providers (Tweets, Telegram, Discord, etc.), though each might have specific fields (api_keys, tokens).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can load a valid `rsudp` configuration (converted to TOML/YAML) and have all ~50+ parameters correctly populated in the application's memory.
- **SC-002**: The application starts successfully with a provided valid config file.
- **SC-003**: The application fails gracefully (clear error message, exit code non-zero) when provided a syntactically invalid config file.
- **SC-004**: Generated default config files are valid and can be immediately read back by the application without modification.