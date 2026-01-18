# Implementation Plan: SNS Posting for Seismic Alerts

**Branch**: `023-sns-posting` | **Date**: 2026-01-18 | **Spec**: [/specs/023-sns-posting/spec.md](/specs/023-sns-posting/spec.md)
**Input**: Feature specification from `/specs/023-sns-posting/spec.md`

## Summary

Implement SNS notification functionality for Discord, LINE, Google Chat, and Amazon SNS, mirroring the features of the original Python `rsudp`. Notifications will be sent in two stages: immediate text notification on trigger, and a summary with a waveform image on reset. Images for LINE and Google Chat will be hosted via Amazon S3.

## Technical Context

**Language/Version**: Rust 1.7x (Edition 2024)
**Primary Dependencies**: 
- `reqwest`: HTTP client for webhooks and Messaging API.
- `aws-sdk-s3`, `aws-sdk-sns`: Official AWS SDKs.
- `aws-config`: AWS credential management.
- `tokio`: Async task management (already in project).
**Storage**: local filesystem (temporary PNGs), Amazon S3 (public image hosting).
**Testing**: Manual integration with webhooks and `streamer` simulation.
**Target Platform**: Linux, macOS, Windows.
**Project Type**: Feature Implementation.
**Performance Goals**: < 10s latency for rich notifications.
**Constraints**: Non-blocking pipeline execution.
**Scale/Scope**: Logic contained within `src/web/sns/` (new module) and integrated into `src/pipeline.rs`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Check | Result |
|-----------|-------|--------|
| I. 安定性と信頼性 | 非同期タスク化によりメインパイプラインを保護する | ✅ |
| II. 厳密なテスト | モックまたは実環境での検証手順を含む | ✅ |
| III. 高いパフォーマンス | `reqwest` と `aws-sdk` の非同期機能を活用する | ✅ |
| IV. 明瞭性と保守性 | トレイトを用いたプロバイダー抽象化を行う | ✅ |
| V. 日本語による仕様策定 | 済み | ✅ |
| VI. 標準技術スタック | Rust標準（reqwest, aws-sdk）を採用 | ✅ |
| VII. 自己検証の義務 | 開発者がSNSへの実投稿を確認する | ✅ |
| VIII. ブランチ運用 | フィーチャーブランチを使用している | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/023-sns-posting/
├── plan.md              # This file
├── research.md          # Technical decisions (crates, patterns)
├── data-model.md        # Entities and Trait definitions
├── quickstart.md        # Configuration guide
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
rsudp-rust/
├── src/
│   ├── web/
│   │   ├── sns/         # New module
│   │   │   ├── mod.rs   # SNSManager
│   │   │   ├── discord.rs
│   │   │   ├── line.rs
│   │   │   ├── gchat.rs
│   │   │   ├── s3.rs
│   │   │   └── aws_sns.rs
│   └── pipeline.rs      # Integration point
```

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

*(No violations detected)*