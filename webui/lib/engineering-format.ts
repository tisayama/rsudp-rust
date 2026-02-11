// SI prefix formatter matching rsudp EngFormatter behavior

const SI_PREFIXES: [number, string][] = [
  [1e9, 'G'],
  [1e6, 'M'],
  [1e3, 'k'],
  [1, ''],
  [1e-3, 'm'],
  [1e-6, 'μ'],
  [1e-9, 'n'],
];

export function formatEngineering(value: number, unit?: string): string {
  if (value === 0) return unit ? `0 ${unit}` : '0';

  const absVal = Math.abs(value);

  for (const [threshold, prefix] of SI_PREFIXES) {
    if (absVal >= threshold * 0.9999) {
      const scaled = value / threshold;
      const formatted = Math.abs(scaled) >= 100
        ? scaled.toFixed(0)
        : Math.abs(scaled) >= 10
          ? scaled.toFixed(1)
          : scaled.toFixed(2);
      const suffix = unit ? `${prefix}${unit}` : prefix;
      return suffix ? `${formatted} ${suffix}` : formatted;
    }
  }

  // Very small values
  const scaled = value / 1e-9;
  const formatted = Math.abs(scaled) >= 10 ? scaled.toFixed(1) : scaled.toFixed(2);
  return unit ? `${formatted} n${unit}` : `${formatted} n`;
}
