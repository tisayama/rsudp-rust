# Tasks: WebUI Plot Polish

**Input**: Design documents from `/specs/037-webui-plot-polish/`
**Prerequisites**: plan.md (required), spec.md (required), research.md

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Create shared utility needed by multiple user stories

- [X] T001 Create 1-2-5 series nice-number utility in webui/lib/nice-number.ts implementing `niceStep(range, targetTicks)` function per research.md R1 algorithm (computes magnitude, normalizes fraction, snaps to 1-2-5 series). Export `niceStep` and `computeNiceTicks(min, max, targetTicks)` which returns an array of tick values.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Timestamp propagation infrastructure needed before time axis can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 Add `latestTimestamp` tracking per channel in webui/src/app/page.tsx: in the Waveform message handler, compute the timestamp of the latest sample as `new Date(Date.parse(timestamp) + (samples.length / sample_rate) * 1000)` and store in a `Record<string, Date>` state. Pass the per-channel timestamp to each ChannelPairCanvas as a new `latestTimestamp` prop.
- [X] T003 Update ChannelPairCanvas interface in webui/components/ChannelPairCanvas.tsx: add `latestTimestamp: Date | null` to `ChannelPairCanvasProps`. Accept the prop in the component destructuring. No rendering changes yet.

**Checkpoint**: Foundation ready — timestamp data flows from page to canvas component.

---

## Phase 3: User Story 1 — Absolute Time Axis Labels (Priority: P1)

**Goal**: Replace relative seconds (0, 10, 20...) with absolute HH:MM:SS (UTC) time labels at 10-second clock-aligned intervals. Time labels appear between waveform and spectrogram; "Time (UTC)" title appears below spectrogram.

**Independent Test**: Open WebUI with live data → X-axis shows HH:MM:SS labels matching current UTC clock at 10s intervals, scrolling with data.

### Implementation

- [X] T004 [US1] Rework canvas height layout in webui/components/ChannelPairCanvas.tsx: when spectrogram is shown, the waveform canvas gets `waveformHeight + TIME_AXIS_HEIGHT` (time labels below waveform border, between plots); spectrogram canvas gets `spectrogramHeight + TIME_LABEL_HEIGHT` (for "Time (UTC)" title below). When spectrogram is hidden, waveform canvas gets `waveformHeight + TIME_AXIS_HEIGHT + TIME_LABEL_HEIGHT`. Update corresponding JSX canvas elements and useEffect dependencies.
- [X] T005 [US1] Replace relative time labels with absolute HH:MM:SS in webui/components/ChannelPairCanvas.tsx: using `latestTimestamp` prop, compute rightEdge (ms) and leftEdge = rightEdge - windowSeconds*1000. Find first 10-second-aligned boundary via `Math.ceil(leftEdge / 10000) * 10000`. Loop at 10000ms intervals, compute x position, format as HH:MM:SS (UTC) using `new Date(t).toISOString().substr(11, 8)`. Draw labels at `waveformHeight + 14` (in the gap area below waveform). Remove old relative-second label code.
- [X] T006 [US1] Add "Time (UTC)" axis title in webui/components/ChannelPairCanvas.tsx: draw centered text "Time (UTC)" below the spectrogram (at `spectrogramHeight + TIME_LABEL_HEIGHT - 2`) or below the waveform time labels area when spectrogram is hidden. Remove old "Time (seconds)" label.

**Checkpoint**: Time axis shows absolute HH:MM:SS labels at 10s intervals, correctly positioned between waveform and spectrogram.

---

## Phase 4: User Story 2 — White Border Frames Around Plots (Priority: P1)

**Goal**: Draw white/light rectangular border frames around the waveform and spectrogram data areas, matching rsudp desktop.

**Independent Test**: Open WebUI → both waveform and spectrogram have visible white rectangular borders framing the data area only (inside axis margins).

### Implementation

- [X] T007 [US2] Draw waveform plot border in webui/components/ChannelPairCanvas.tsx: after all waveform rendering (line, labels, alerts), draw a `strokeRect` with `strokeStyle = 'rgba(255, 255, 255, 0.6)'`, `lineWidth = 1`, rect at `(LEFT_MARGIN, 0, plotWidth, waveformHeight)`. Use `0.5` pixel offsets for crisp 1px lines.
- [X] T008 [US2] Draw spectrogram plot border in webui/components/ChannelPairCanvas.tsx: after spectrogram rendering and Hz labels, draw a `strokeRect` with same style, rect at `(LEFT_MARGIN, 0, plotWidth, spectrogramHeight)`.

**Checkpoint**: Both plot areas have white borders matching the reference screenshot.

---

## Phase 5: User Story 3 — Y-Axis Tick Spacing with Round Numbers (Priority: P1)

**Goal**: Replace irregular Y-axis ticks with evenly-spaced round numbers (1-2-5 series), add horizontal grid lines, use actual data range (non-symmetric).

**Independent Test**: Open WebUI with live data → Y-axis shows round numbers (e.g., -2, -1, 0, 1, 2) with even spacing and faint horizontal grid lines at each tick.

### Implementation

- [X] T009 [US3] Change Y-axis range from symmetric to actual min/max in webui/components/ChannelPairCanvas.tsx: replace the current `yMin = -maxAbs * 1.1, yMax = maxAbs * 1.1` auto-scale logic with actual data min/max computation (after DC offset removal). Import `computeNiceTicks` from `nice-number.ts`, compute tick values, and set yMin/yMax to the first/last tick value (expanded to nice boundaries).
- [X] T010 [US3] Replace fixed 5-tick Y-axis labels with nice-number ticks in webui/components/ChannelPairCanvas.tsx: replace the `for (let i = 0; i <= yTicks; i++)` loop with iteration over the tick array from `computeNiceTicks`. Draw each tick label at the correct y position via `mapY(tickValue)`. Continue using `formatEngineering(val)` for label formatting.
- [X] T011 [US3] Add faint horizontal grid lines at Y-axis tick positions in webui/components/ChannelPairCanvas.tsx: for each tick value from `computeNiceTicks`, draw a horizontal line from `(LEFT_MARGIN, mapY(tick))` to `(LEFT_MARGIN + plotWidth, mapY(tick))` with `strokeStyle = 'rgba(255, 255, 255, 0.15)'`, `lineWidth = 1`. Draw before the waveform line so grid is behind data.

**Checkpoint**: Y-axis shows round-number ticks with grid lines, matching reference screenshot.

---

## Phase 6: User Story 4 — Remove Alert History (Priority: P2)

**Goal**: Remove the /history page, navigation link, and unused AlertEvent type. Keep useAlerts.ts and VisualAlertMarker.

**Independent Test**: No "View Alert History" button visible; /history returns 404; audio alerts and waveform alert markers still work.

### Implementation

- [X] T012 [P] [US4] Delete the Alert History page at webui/src/app/history/page.tsx (remove entire file and history/ directory)
- [X] T013 [P] [US4] Remove "View Alert History" Link from webui/src/app/page.tsx: delete the `<Link href="/history">` element (lines 254-259) and remove the `import Link from 'next/link'` import statement
- [X] T014 [P] [US4] Remove unused `AlertEvent` interface from webui/lib/types.ts (lines 28-36). Verify no other files import it before removal.

**Checkpoint**: No traces of Alert History in UI or code. Audio alerts and waveform markers still functional.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Minor improvements and verification

- [X] T015 Change spectrogram frequency axis label from `'Hz'` to `'Frequency (Hz)'` in the rotated label section of webui/components/ChannelPairCanvas.tsx (around line 259 `sCtx.fillText('Hz', 0, 0)`)
- [ ] T016 Visual verification: run the WebUI with live data and verify all 6 quickstart.md scenarios pass by comparing against `specs/037-webui-plot-polish/references/R6E01-2025-11-25-090123.png`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: No dependencies — can run in parallel with Phase 1
- **US1 (Phase 3)**: Depends on Phase 2 (needs latestTimestamp prop)
- **US2 (Phase 4)**: Depends on Phase 3 (canvas layout changes must be done first)
- **US3 (Phase 5)**: Depends on Phase 1 (needs nice-number.ts); can run after Phase 4
- **US4 (Phase 6)**: Independent — can run in parallel with any phase after Setup
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends on Phase 2 (latestTimestamp). No dependency on other stories.
- **US2 (P1)**: Should run after US1 (canvas layout changes). No dependency on US3/US4.
- **US3 (P1)**: Depends on Phase 1 (nice-number.ts). Should run after US2 (same file, sequential safety).
- **US4 (P2)**: Fully independent. Can run at any point.

### Within Each User Story

- Implementation tasks within each story are sequential (same file modifications)
- US4 tasks marked [P] can all run in parallel (different files)

### Parallel Opportunities

- T001 (Setup) and T002+T003 (Foundational) can run in parallel
- T012, T013, T014 (US4 tasks) can all run in parallel with each other
- US4 (Phase 6) can run in parallel with US1/US2/US3 phases

---

## Parallel Example: Setup + Foundational

```bash
# These can run simultaneously:
Task: "T001 - Create nice-number.ts utility in webui/lib/nice-number.ts"
Task: "T002 - Add latestTimestamp tracking in webui/src/app/page.tsx"
```

## Parallel Example: US4 Alert History Removal

```bash
# All three can run simultaneously (different files):
Task: "T012 - Delete webui/src/app/history/page.tsx"
Task: "T013 - Remove Link from webui/src/app/page.tsx"
Task: "T014 - Remove AlertEvent from webui/lib/types.ts"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (nice-number.ts) + Phase 2: Foundational (timestamp propagation)
2. Complete Phase 3: US1 (absolute time axis)
3. **STOP and VALIDATE**: Time labels show HH:MM:SS at 10s intervals
4. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (Absolute Time) → Verify → Most impactful visual change done
3. US2 (Borders) → Verify → Plot areas clearly delineated
4. US3 (Y-Axis Ticks) → Verify → All axis improvements complete
5. US4 (Alert History Removal) → Verify → Dead code removed
6. Polish → Final visual verification against reference screenshot

---

## Notes

- All US1/US2/US3 changes are in webui/components/ChannelPairCanvas.tsx — execute sequentially to avoid merge conflicts
- US4 is the only story that can safely run in parallel with others (different files)
- T001 creates nice-number.ts as an independent utility — can be unit tested in isolation
- Visual verification (T016) requires Docker + live data streaming
