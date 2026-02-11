import { INFERNO_RGB } from './inferno-colormap';

export class SpectrogramRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private frequencyBins: number;
  private columnCount: number = 0;

  constructor(canvas: HTMLCanvasElement, frequencyBins: number) {
    this.canvas = canvas;
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Failed to get 2d context');
    this.ctx = ctx;
    this.frequencyBins = frequencyBins;
    this.clear();
  }

  get maxColumns(): number {
    return this.canvas.width;
  }

  addColumn(u8Data: Uint8Array): void {
    if (u8Data.length !== this.frequencyBins) return;

    const { ctx, canvas } = this;
    const w = canvas.width;
    const h = canvas.height;

    // Shift existing content left by 1 pixel
    if (this.columnCount > 0) {
      ctx.drawImage(canvas, 1, 0, w - 1, h, 0, 0, w - 1, h);
    }

    // Draw new column at right edge
    const imageData = ctx.createImageData(1, h);
    const pixels = imageData.data;

    for (let row = 0; row < h; row++) {
      // Map canvas row to frequency bin (bottom = 0 Hz, top = Nyquist)
      const binIndex = Math.floor(((h - 1 - row) / (h - 1)) * (this.frequencyBins - 1));
      const value = u8Data[Math.min(binIndex, this.frequencyBins - 1)];
      const clampedIdx = Math.max(0, Math.min(INFERNO_RGB.length - 1, value));
      const [r, g, b] = INFERNO_RGB[clampedIdx];
      const pixelOffset = row * 4;
      pixels[pixelOffset] = r;
      pixels[pixelOffset + 1] = g;
      pixels[pixelOffset + 2] = b;
      pixels[pixelOffset + 3] = 255;
    }

    ctx.putImageData(imageData, w - 1, 0);
    this.columnCount = Math.min(this.columnCount + 1, w);
  }

  addBulkColumns(columns: Uint8Array[]): void {
    if (!columns || columns.length === 0) return;

    const { ctx, canvas } = this;
    const w = canvas.width;
    const h = canvas.height;

    // If more columns than canvas width, only use the last w columns
    const startIdx = Math.max(0, columns.length - w);
    const visibleColumns = columns.slice(startIdx);

    // Clear and draw all columns at once
    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, w, h);

    const xOffset = w - visibleColumns.length;

    for (let col = 0; col < visibleColumns.length; col++) {
      const u8Data = visibleColumns[col];
      if (!u8Data || u8Data.length !== this.frequencyBins) continue;

      const imageData = ctx.createImageData(1, h);
      const pixels = imageData.data;

      for (let row = 0; row < h; row++) {
        const binIndex = Math.floor(((h - 1 - row) / (h - 1)) * (this.frequencyBins - 1));
        const value = u8Data[Math.min(binIndex, this.frequencyBins - 1)];
        const clampedIdx = Math.max(0, Math.min(INFERNO_RGB.length - 1, value));
        const [r, g, b] = INFERNO_RGB[clampedIdx];
        const pixelOffset = row * 4;
        pixels[pixelOffset] = r;
        pixels[pixelOffset + 1] = g;
        pixels[pixelOffset + 2] = b;
        pixels[pixelOffset + 3] = 255;
      }

      ctx.putImageData(imageData, xOffset + col, 0);
    }

    this.columnCount = visibleColumns.length;
  }

  clear(): void {
    const { ctx, canvas } = this;
    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    this.columnCount = 0;
  }

  setFrequencyBins(bins: number): void {
    this.frequencyBins = bins;
  }
}
