# Research: Cleanup Project Files and Update README

## Decisions

### Decision: Removal of nested `rsudp-rust/rsudp-rust/`
**Rationale**: This appears to be an artifact from an incorrect cargo initialization or a manual copy. It contains a `target` directory and clutters the project structure.
**Rationale**: 誤った cargo の初期化または手動コピーによる残骸と思われます。`target` ディレクトリを含んでおり、プロジェクト構造を乱雑にしています。

### Decision: Updating `.gitignore`
**Rationale**: To prevent `rsudp.pid` and test images from being committed again, patterns like `*.pid` and `alerts/*.png` (excluding `.gitkeep`) should be added to the root or sub-directory `.gitignore`.
**Rationale**: `rsudp.pid` やテスト画像が再びコミットされるのを防ぐため、`*.pid` や `alerts/*.png`（`.gitkeep` を除く）などのパターンを `.gitignore` に追加します。

### Decision: README Content Strategy
**Rationale**: The README will serve as the primary entry point. It needs to explain the Rust implementation's benefits (performance, safety) and provide clear commands for the core functionalities developed so far (UDP receiving, intensity calculation, alerting).
**Rationale**: README は主要なエントリポイントとなります。Rust 実装の利点（パフォーマンス、安全性）を説明し、これまでに開発された主要機能（UDP受信、震度計算、アラート）の明確なコマンドを提供する必要があります。

## Research Tasks

- [x] Identify all temporary files in `rsudp-rust/`
- [x] Draft README content in English and Japanese
