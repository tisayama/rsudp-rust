/**
 * 1-2-5 series "nice number" tick algorithm (Heckbert, 1990).
 * Produces human-readable axis tick values for scientific plots.
 */

/**
 * Compute a "nice" step size that falls on a 1-2-5 series boundary.
 * @param range The data range (max - min)
 * @param targetTicks Desired number of ticks (typically 5)
 * @returns A step size that is a multiple of 1, 2, or 5 at the appropriate magnitude
 */
export function niceStep(range: number, targetTicks: number): number {
  if (range <= 0 || targetTicks <= 0) return 1;

  const roughStep = range / targetTicks;
  const magnitude = Math.pow(10, Math.floor(Math.log10(roughStep)));
  const fraction = roughStep / magnitude;

  let niceFraction: number;
  if (fraction <= 1.5) {
    niceFraction = 1;
  } else if (fraction <= 3.5) {
    niceFraction = 2;
  } else if (fraction <= 7.5) {
    niceFraction = 5;
  } else {
    niceFraction = 10;
  }

  return niceFraction * magnitude;
}

/**
 * Compute an array of "nice" tick values spanning [min, max].
 * The returned ticks are evenly spaced at a 1-2-5 series interval,
 * and the first/last ticks may extend slightly beyond the data range.
 *
 * @param min Data minimum
 * @param max Data maximum
 * @param targetTicks Desired number of ticks (default 5)
 * @returns Array of tick values (always at least 2 values)
 */
export function computeNiceTicks(min: number, max: number, targetTicks: number = 5): number[] {
  if (min >= max) return [min, max];

  const range = max - min;
  const step = niceStep(range, targetTicks);

  const tickMin = Math.floor(min / step) * step;
  const tickMax = Math.ceil(max / step) * step;

  const ticks: number[] = [];
  // Use epsilon to avoid floating-point overshoot
  for (let v = tickMin; v <= tickMax + step * 0.001; v += step) {
    // Round to avoid floating-point artifacts (e.g., 0.30000000000000004)
    const rounded = Math.round(v / step) * step;
    ticks.push(rounded);
  }

  return ticks;
}
