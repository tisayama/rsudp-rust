/**
 * Matplotlib-compatible path simplification algorithm.
 *
 * Port of matplotlib's PathSimplifier from src/path_converters.h.
 * Uses perpendicular-distance-based streaming merge with forward/backward
 * extreme tracking. This produces stable, jitter-free rendering because
 * point selection is geometry-driven (not pixel-bin or step-based).
 *
 * Reference: https://github.com/matplotlib/matplotlib/blob/main/src/path_converters.h
 */

/**
 * Simplify a path in pixel-coordinate space using matplotlib's algorithm.
 *
 * Draws directly to the provided Canvas context (moveTo/lineTo) to avoid
 * intermediate array allocations on every frame.
 *
 * @param ctx - Canvas 2D context to draw the simplified path into
 * @param xs  - X coordinates in pixel space (must be length >= count)
 * @param ys  - Y coordinates in pixel space (must be length >= count)
 * @param count - Number of valid points in xs/ys
 * @param threshold - Perpendicular distance threshold in pixels (default 1/9)
 */
export function drawSimplifiedPath(
  ctx: CanvasRenderingContext2D,
  xs: Float64Array,
  ys: Float64Array,
  count: number,
  threshold: number = 1 / 9,
): void {
  if (count === 0) return;

  const threshSq = threshold * threshold;

  // Track last emitted point for continuity
  let emitCount = 0;
  let lastEmitX = 0;
  let lastEmitY = 0;

  // Emit a vertex to the Canvas path
  const emit = (x: number, y: number) => {
    if (emitCount === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
    lastEmitX = x;
    lastEmitY = y;
    emitCount++;
  };

  if (count === 1) {
    emit(xs[0], ys[0]);
    return;
  }

  // PathSimplifier state
  let origdx = 0;       // Reference direction vector X
  let origdy = 0;       // Reference direction vector Y
  let origdNorm2 = 0;   // Squared norm of reference direction
  let startX = 0;       // Start of current merged segment (= last emitted point)
  let startY = 0;
  let lastx = xs[0];    // Last processed point
  let lasty = ys[0];

  // Forward extreme (along reference direction)
  let fwdMax = 0;
  let fwdX = 0;
  let fwdY = 0;
  let wasFwd = false;    // Was the last merged point the forward extreme?

  // Backward extreme (anti-parallel to reference direction)
  let bwdMax = 0;
  let bwdX = 0;
  let bwdY = 0;
  let wasBwd = false;    // Was the last merged point the backward extreme?

  // Flush accumulated extremes to the Canvas path
  const flush = () => {
    if (bwdMax > 0) {
      // We saw movement in both directions: emit both extremes
      if (wasFwd) {
        // Last extreme was forward → emit backward first, then forward
        emit(bwdX, bwdY);
        emit(fwdX, fwdY);
      } else {
        emit(fwdX, fwdY);
        emit(bwdX, bwdY);
      }
    } else {
      // Only forward movement: emit the forward extreme
      emit(fwdX, fwdY);
    }

    // If the most recent point wasn't itself an extreme, emit it
    // to maintain proper continuity with the next segment
    if (!wasFwd && !wasBwd) {
      emit(lastx, lasty);
    }
  };

  // Always emit the first point
  emit(lastx, lasty);

  for (let i = 1; i < count; i++) {
    const x = xs[i];
    const y = ys[i];

    // If we don't have a reference vector yet, establish one
    if (origdNorm2 === 0) {
      origdx = x - lastx;
      origdy = y - lasty;
      origdNorm2 = origdx * origdx + origdy * origdy;

      if (origdNorm2 === 0) {
        // Duplicate point — skip
        lastx = x;
        lasty = y;
        continue;
      }

      startX = lastx;
      startY = lasty;
      fwdMax = origdNorm2;
      fwdX = x;
      fwdY = y;
      wasFwd = true;
      bwdMax = 0;
      wasBwd = false;
      lastx = x;
      lasty = y;
      continue;
    }

    // Compute perpendicular distance from the reference direction line
    const totdx = x - startX;
    const totdy = y - startY;
    const dot = origdx * totdx + origdy * totdy;
    const parX = (dot * origdx) / origdNorm2;
    const parY = (dot * origdy) / origdNorm2;
    const perpX = totdx - parX;
    const perpY = totdy - parY;
    const perpSq = perpX * perpX + perpY * perpY;

    if (perpSq < threshSq) {
      // Within threshold — merge this point, but track extremes
      const parSq = parX * parX + parY * parY;
      wasFwd = false;
      wasBwd = false;

      if (dot > 0) {
        // Forward direction (parallel to reference)
        if (parSq > fwdMax) {
          fwdMax = parSq;
          fwdX = x;
          fwdY = y;
          wasFwd = true;
        }
      } else {
        // Backward direction (anti-parallel to reference)
        if (parSq > bwdMax) {
          bwdMax = parSq;
          bwdX = x;
          bwdY = y;
          wasBwd = true;
        }
      }

      lastx = x;
      lasty = y;
      continue;
    }

    // Perpendicular distance exceeded threshold — flush and start new segment
    flush();

    // Start new reference vector
    origdx = x - lastx;
    origdy = y - lasty;
    origdNorm2 = origdx * origdx + origdy * origdy;
    startX = lastEmitX;
    startY = lastEmitY;
    fwdMax = origdNorm2;
    fwdX = x;
    fwdY = y;
    wasFwd = true;
    bwdMax = 0;
    wasBwd = false;
    lastx = x;
    lasty = y;
  }

  // Final flush for the last accumulated segment
  if (origdNorm2 > 0) {
    flush();
  }

  // Ensure the very last data point is emitted for path completeness
  if (emitCount > 0 && (lastEmitX !== lastx || lastEmitY !== lasty)) {
    emit(lastx, lasty);
  }
}
