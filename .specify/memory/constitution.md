<!--
SYNC IMPACT REPORT
- Version: 1.0.0 → 1.1.0 (MINOR bump)
- Change: Added standard technology stack requirements for WebUI.
- Principles Added:
  - VI. 標準技術スタック (Standard Tech Stack)
- Templates Checked:
  - .specify/templates/plan-template.md (✅ No changes needed, logic already covers constitution checks)
  - .specify/templates/spec-template.md (✅ No changes needed)
  - .specify/templates/tasks-template.md (✅ No changes needed)
-->
# rsudp-rust 憲法

## 基本原則 (Core Principles)

### I. 安定性と信頼性 (Stability and Reliability)
重大な地震データとアラートを扱うため、システムは非常に安定かつ信頼性が高くなければなりません。すべてのコードは堅牢で、エラー処理が徹底され、予期せぬ入力やシステム状態に対して回復力を持つ必要があります。

### II. 厳密なテスト (Rigorous Testing)
すべてのコンポーネントには、単体テスト、結合テストを含む包括的なテストが伴わなければなりません。計算の正確性を保証するため、テストデータは適切に管理され、現実世界のシナリオを反映する必要があります。これは譲歩不可能な要件です。

### III. 高いパフォーマンス (High Performance)
PythonアプリケーションのRustによる書き直しとして、大幅なパフォーマンス向上が重要な目標です。コードは、明瞭性や保守性を犠牲にすることなく、速度と低リソース消費のために最適化されるべきです。

### IV. コードの明瞭性と保守性 (Clarity and Maintainability)
コードは、明確で、慣用的なRustのスタイルで書かれなければなりません。ドキュメント、コメント、命名規則は、将来の開発者がコードベースを容易に理解し、保守できるように整備されるべきです。

### V. 日本語による仕様策定 (Specification in Japanese)
プロジェクトの関係者との明確なコミュニケーションと整合性を確保するため、すべてのプロジェクト仕様書、設計ドキュメント、および要件明確化に関する議論は日本語で行われなければなりません。

### VI. 標準技術スタック (Standard Tech Stack)
WebUIを実装する場合、通信にはREST APIおよびWebSockets（サーバサイド: Rust）を使用し、フロントエンドはNext.jsおよびTailwind CSSを使用しなければなりません。これにより、開発の一貫性と保守性を確保します。

## 開発ワークフロー (Development Workflow)

本プロジェクトでは、仕様駆動開発（Specification-Driven Development）を採用します。新しい機能の実装や既存機能の変更は、必ず事前に仕様書を作成し、関係者の合意を得てから着手します。

## 品質ゲート (Quality Gates)

すべてのコード変更は、プルリクエストを通じてレビューされる必要があります。マージされるためには、以下の条件を満たさなければなりません。
1.  すべてのテストが成功すること。
2.  コードカバレッジが低下しないこと。
3.  定義されたコーディング規約に準拠していること。

## ガバナンス (Governance)

この憲法は、他のすべての慣行や規約に優先します。憲法の改正には、変更内容の文書化、主要な貢献者による承認、そして必要に応じた既存コードベースへの移行計画が必要です。すべてのプルリクエストとコードレビューは、この憲法の原則に準拠しているか検証する必要があります。

**バージョン**: 1.1.0 | **制定日**: 2025-12-18 | **最終改正日**: 2025-12-19