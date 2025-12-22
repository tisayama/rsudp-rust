# Research: Intensity Inclusion in Alert Messages

## Decisions & Rationale

### 1. Message Storage and Delivery
- **Decision**: Generate and store the descriptive Japanese message in the `AlertEvent` entity within the `pipeline.rs` logic.
- **Rationale**: By generating the message at the point where max intensity is finalized (Reset event), we ensure consistency across all notification channels (WebUI, Email, Logs) without duplicating the formatting logic.
- **Alternatives considered**: 
    - Format on-the-fly in the WebUI and Email sender: Rejected because it requires duplicating the "震度X相当" logic in both Rust and TypeScript.
    - Store only the raw intensity: Rejected because the user specifically requested specific phrasing ("揺れを検出できませんでした" vs "震度X相当の揺れを検出しました").

### 2. Phrasing Mapping
- **Decision**: Replicate the "弱/強" logic from the Plot Badge (Feature 012).
- **Mapping**:
    - Intensity < 0.5 (Class 0): "揺れを検出できませんでした"
    - Intensity >= 0.5: "震度 {Class}相当の揺れを検出しました"
    - Classes with +/-: "5-" -> "5弱", "5+" -> "5強", "6-" -> "6弱", "6+" -> "6強".

### 3. Integration Point
- **Decision**: Update the `Reset` logic in `pipeline.rs` to compute this string and include it in the `AlertEnd` broadcast and the history update.
