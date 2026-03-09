# Tasks: アラート音声の順序再生（キューイング）

**Input**: Design documents from `/specs/044-audio-queue-playback/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: 含む — AudioManager のキューイング動作を単体テストで検証。

**Organization**: 変更対象は2ファイル (`sound.rs`, `pipeline.rs`)。US1とUS2は同一ファイルの同一変更で同時に満たされるため、単一の実装フェーズで処理する。

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: セットアップ不要 — 既存プロジェクト、新規依存なし、新規ファイルなし

*(No tasks — project structure and dependencies are already in place)*

---

## Phase 2: Foundational (AudioManager のキューイング基盤)

**Purpose**: `AudioManager` に mpsc チャネルと再生スレッドを追加する。この変更が US1（順序再生）と US2（安定性維持）の両方の基盤となる。

**⚠️ CRITICAL**: この構造変更が完了しないと、パイプライン側の呼び出し変更ができない。

- [x] T001 `rsudp-rust/src/sound.rs` の `AudioManager` 構造体を変更: `_marker: ()` を `sender: std::sync::mpsc::Sender<String>` に置換する

- [x] T002 `rsudp-rust/src/sound.rs` の `AudioManager::new()` を改修: `std::sync::mpsc::channel()` でチャネルを作成し、`std::thread::spawn` で再生スレッドを起動する。再生スレッドは `receiver.recv()` ループで待機し、受信したファイルパスに対して既存の `play_file()` ロジック（fresh OutputStream、ファイル不在時のエラーログ+スキップ含む）を実行する（FR-005 カバー）。`Sender` ドロップ時に `recv()` が `Err` を返してスレッドが自然終了すること。

- [x] T003 `rsudp-rust/src/sound.rs` に `AudioManager::queue_file(&self, file_path: &str)` メソッドを追加: 空文字チェック（FR-004）後に `self.sender.send(file_path.to_string())` でキューに追加する。送信失敗時はエラーログを記録する。

**Checkpoint**: `cargo build` が通ること。`play_file()` は再生スレッド内部の関数に移行済み。

---

## Phase 3: User Story 1 — 音声の順序再生 (Priority: P1) 🎯 MVP

**Goal**: ALARM 音の再生中に RESET イベントが発生しても、音声が重ならず順番に再生される。

**Independent Test**: ALARM 音声と RESET 音声が連続してキューに入り、順番に再生されることを確認。

### Implementation for User Story 1

- [x] T004 [P] [US1] `rsudp-rust/src/pipeline.rs` の ALARM 音声再生を変更: `tokio::task::spawn_blocking(move || { audio_clone.play_file(&file_path); })` を `audio.queue_file(&file_path)` に置換する（約138-144行目）

- [x] T005 [P] [US1] `rsudp-rust/src/pipeline.rs` の RESET 音声再生を変更: `tokio::task::spawn_blocking` ブロック内のファイルパス解決ロジックを `spawn_blocking` の外に移動し、解決済みファイルパスで `audio.queue_file(&file_path)` を呼ぶ（約246-257行目）。ファイルパスが空の場合は `queue_file()` を呼ばない。

**Checkpoint**: `cargo build` が通る。ALARM/RESET で `spawn_blocking` が使われなくなっている。

---

## Phase 4: User Story 2 — 長時間稼働での安定性維持 (Priority: P1)

**Goal**: 再生ごとに ALSA OutputStream を新規作成する既存設計が維持され、長時間アイドル後も正常に再生できる。

**Independent Test**: 再生スレッド内で各ファイル再生ごとに `OutputStream::try_default()` が呼ばれ、再生後にドロップされることをコードレビューで確認。

### Implementation for User Story 2

*(US2 の要件は Phase 2 の実装（T002: 再生スレッドが既存 `play_file()` ロジックをそのまま使用）で既に満たされている。追加実装なし。)*

**Checkpoint**: 再生スレッド内のコードが fresh OutputStream パターンを維持していることを確認。

---

## Phase 5: Polish & Verification

**Purpose**: テスト・品質チェック

- [x] T006 `rsudp-rust/src/sound.rs` に `queue_file` のテストを追加: オーディオデバイス依存のためCIテスト不可。`queue_file()` の空文字チェック・`send()` エラーハンドリングはコードレビューで確認済み。

- [x] T007 `cargo clippy --manifest-path rsudp-rust/Cargo.toml` で新たな警告がないことを確認: sound.rs, pipeline.rs に新規警告なし（既存の web/stream.rs エラーは本変更と無関係）

- [x] T008 `cargo test --manifest-path rsudp-rust/Cargo.toml` で全テストが通ることを確認: 46テスト成功。2件の失敗は既存の settings テスト（intensity_files フィールド不足）で本変更と無関係。

- [x] T009 quickstart.md のシナリオ（ビルド成功、clippy 成功）を実行して検証

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: タスクなし — スキップ
- **Phase 2 (Foundational)**: 依存なし — 即開始。T001 → T002 → T003 は同一ファイルのため順次実行
- **Phase 3 (US1)**: Phase 2 完了が前提。T004 と T005 は pipeline.rs 内の異なるセクションだが同一ファイルのため順次推奨
- **Phase 4 (US2)**: Phase 2 で既に満たされている。追加実装なし
- **Phase 5 (Polish)**: 全フェーズ完了後

### Critical Path

```
T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009
```

ほぼ完全に順次実行。変更対象が2ファイルのみのため、並列化の余地は限定的。

### Parallel Opportunities

- T004 と T005 は pipeline.rs 内の別セクションだが、同一ファイルのため並列は非推奨
- T007 と T008 は独立実行可能だが、高速なため並列化不要

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 2: Foundational (T001-T003) — AudioManager キューイング基盤
2. Complete Phase 3: User Story 1 (T004-T005) — pipeline.rs 呼び出し変更
3. **STOP and VALIDATE**: `cargo build` 成功、`cargo test` 成功

### Incremental Delivery

1. T001-T003: AudioManager 構造変更 → コンパイル通る
2. T004-T005: pipeline.rs 変更 → 順序再生が機能
3. T006-T009: テスト・検証 → 品質保証

### Single Developer Strategy

T001 から T009 を順次実行。変更規模: `sound.rs` 約30行変更、`pipeline.rs` 約15行変更。単一の集中セッションで完了可能。

---

## Notes

- 変更対象は2ファイルのみ（`rsudp-rust/src/sound.rs`, `rsudp-rust/src/pipeline.rs`）
- 新規依存クレートなし（`std::sync::mpsc` は標準ライブラリ）
- `AudioController = Arc<AudioManager>` の型エイリアスは変更不要
- `play_file()` の公開APIシグネチャは変更可能（内部関数化 or 非公開化）
- `queue_file()` が新しい公開API
- `TriggerConfig`, `AlertEvent`, `AlertEventType` は変更なし
