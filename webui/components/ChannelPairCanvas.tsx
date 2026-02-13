'use client';

import React, { useEffect, useRef, useState, useLayoutEffect } from 'react';
import { RingBuffer } from '../lib/RingBuffer';
import { VisualAlertMarker, PlotSettings } from '../lib/types';
import { INFERNO_RGB } from '../lib/inferno-colormap';
import { formatEngineering } from '../lib/engineering-format';
import { computeNiceTicks } from '../lib/nice-number';
import { drawSimplifiedPath } from '../lib/path-simplifier';

interface ChannelPairCanvasProps {
  channelId: string;
  buffer: RingBuffer;
  spectrogramColumns: Uint8Array[];
  frequencyBins: number;
  sampleRate: number;
  hopDuration: number;
  windowSeconds: number;
  autoScale: boolean;
  alerts: VisualAlertMarker[];
  settings: PlotSettings;
  isBottomChannel: boolean;
  units: string;
  latestTimestamp: Date | null;
  channelLatestTimestamp: Date | null;
  spectrogramFirstColumnTimestamp: number;
}

const BG_COLOR = '#202530';
const WAVEFORM_COLOR = '#c28285';
const FG_COLOR = 'rgba(204, 204, 204, 1.0)';
const TRIGGER_COLOR = '#4C8BF5';
const RESET_COLOR = '#D72638';
const LEFT_MARGIN = 70;
const RIGHT_MARGIN = 20;
const TIME_AXIS_HEIGHT = 24;
const TIME_LABEL_HEIGHT = 16;
const SPEC_TOP_PAD = 8;
const BORDER_COLOR = 'rgba(255, 255, 255, 0.6)';
const GRID_COLOR = 'rgba(255, 255, 255, 0.15)';

function getUnitLabel(units: string, channelId: string): string {
  switch (units.toUpperCase()) {
    case 'VEL': return 'Velocity (m/s)';
    case 'ACC': return 'Acceleration (m/s\u00B2)';
    case 'GRAV': return 'Earth gravity (g)';
    case 'DISP': return 'Displacement (m)';
    case 'CHAN': {
      // SEED instrument code (2nd character) determines physical quantity
      const inst = channelId.length >= 2 ? channelId[1] : '';
      if (inst === 'H' || inst === 'L') return 'Velocity (m/s)';   // Seismometer
      if (inst === 'N') return 'Acceleration (m/s\u00B2)';               // Accelerometer
      if (inst === 'G') return 'Acceleration (m/s\u00B2)';               // Gravimeter
      return 'Counts';
    }
    default: return 'Counts';
  }
}

const ChannelPairCanvas: React.FC<ChannelPairCanvasProps> = ({
  channelId,
  buffer,
  spectrogramColumns,
  frequencyBins,
  sampleRate,
  hopDuration,
  windowSeconds,
  autoScale,
  alerts,
  settings,
  isBottomChannel,
  units,
  latestTimestamp,
  channelLatestTimestamp,
  spectrogramFirstColumnTimestamp,
}) => {
  const waveformCanvasRef = useRef<HTMLCanvasElement>(null);
  const spectrogramCanvasRef = useRef<HTMLCanvasElement>(null);

  // Use refs for data that changes frequently to avoid recreating the interval
  const spectrogramColumnsRef = useRef<Uint8Array[]>([]);
  const hopDurationRef = useRef(0.13);
  const frequencyBinsRef = useRef(65);
  spectrogramColumnsRef.current = spectrogramColumns;
  hopDurationRef.current = hopDuration;
  frequencyBinsRef.current = frequencyBins;

  const latestTimestampRef = useRef<Date | null>(null);
  latestTimestampRef.current = latestTimestamp;

  const channelLatestTimestampRef = useRef<Date | null>(null);
  channelLatestTimestampRef.current = channelLatestTimestamp;

  const spectrogramFirstColumnTimestampRef = useRef(0);
  spectrogramFirstColumnTimestampRef.current = spectrogramFirstColumnTimestamp;

  const containerRef = useRef<HTMLDivElement>(null);
  const [canvasWidth, setCanvasWidth] = useState(900); // fallback until measured

  // Measure actual container width so canvas internal resolution = CSS display size.
  // This ensures 1 canvas pixel = 1 CSS pixel, matching matplotlib's 1:1 device-pixel rendering.
  useLayoutEffect(() => {
    const el = containerRef.current;
    if (el) {
      const w = Math.round(el.clientWidth);
      if (w > 0) setCanvasWidth(w);
    }
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const ro = new ResizeObserver(entries => {
      for (const e of entries) {
        const w = Math.round(e.contentRect.width);
        if (w > 0) setCanvasWidth(w);
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const showSpectrogram = settings.show_spectrogram;
  const waveformHeight = 200;
  const spectrogramHeight = showSpectrogram ? 100 : 0;

  // T004: Canvas height layout
  // When spectrogram shown: waveform canvas = data + TIME_AXIS_HEIGHT (HH:MM:SS labels between plots)
  // Spectrogram canvas = data + TIME_LABEL_HEIGHT (for "Time (UTC)" title below)
  // When spectrogram hidden: waveform canvas = data + TIME_AXIS_HEIGHT + TIME_LABEL_HEIGHT
  const waveformCanvasHeight = showSpectrogram
    ? waveformHeight + TIME_AXIS_HEIGHT
    : waveformHeight + TIME_AXIS_HEIGHT + TIME_LABEL_HEIGHT;
  const spectrogramCanvasHeight = showSpectrogram
    ? SPEC_TOP_PAD + spectrogramHeight + TIME_LABEL_HEIGHT
    : 0;

  // Combined waveform + spectrogram rendering at 30 FPS
  useEffect(() => {
    const waveformCanvas = waveformCanvasRef.current;
    if (!waveformCanvas) return;

    const wCtx = waveformCanvas.getContext('2d');
    if (!wCtx) return;

    const render = () => {
      const plotWidth = canvasWidth - LEFT_MARGIN - RIGHT_MARGIN;
      const samples = buffer.getTail(windowSeconds * sampleRate);

      // Global time edges for synchronized rendering
      const globalTs = latestTimestampRef.current;
      const rightEdge = globalTs ? globalTs.getTime() : 0;
      const leftEdge = rightEdge - windowSeconds * 1000;
      const channelTs = channelLatestTimestampRef.current;

      // --- Waveform ---
      wCtx.fillStyle = BG_COLOR;
      wCtx.fillRect(0, 0, canvasWidth, waveformCanvasHeight);

      if (samples.length > 0 && rightEdge > 0 && channelTs) {
        // DC offset removal
        let sum = 0;
        for (let i = 0; i < samples.length; i++) sum += samples[i];
        const mean = sum / samples.length;

        // T009: Auto-scale using actual min/max with nice-number expansion
        let yMin = -1000;
        let yMax = 1000;
        let ticks: number[] = [];
        if (autoScale) {
          let dataMin = Infinity;
          let dataMax = -Infinity;
          for (let i = 0; i < samples.length; i++) {
            const v = samples[i] - mean;
            if (v < dataMin) dataMin = v;
            if (v > dataMax) dataMax = v;
          }
          if (dataMin < dataMax) {
            ticks = computeNiceTicks(dataMin, dataMax, 5);
            yMin = ticks[0];
            yMax = ticks[ticks.length - 1];
          } else {
            ticks = computeNiceTicks(yMin, yMax, 5);
          }
        } else {
          ticks = computeNiceTicks(yMin, yMax, 5);
        }

        const mapY = (val: number) => {
          return waveformHeight - ((val - yMin) / (yMax - yMin)) * waveformHeight;
        };

        // T011: Horizontal grid lines at Y-axis tick positions (drawn before waveform line)
        wCtx.strokeStyle = GRID_COLOR;
        wCtx.lineWidth = 1;
        for (const tick of ticks) {
          const y = Math.round(mapY(tick)) + 0.5;
          wCtx.beginPath();
          wCtx.moveTo(LEFT_MARGIN, y);
          wCtx.lineTo(LEFT_MARGIN + plotWidth, y);
          wCtx.stroke();
        }

        // Draw waveform using matplotlib-compatible path simplification
        // matplotlib lw=0.45 is in POINTS (1/72 inch). At 100 DPI: 0.45 * 100/72 = 0.625 device pixels.
        // All samples are transformed to pixel coordinates, then simplified using
        // matplotlib's perpendicular-distance algorithm with forward/backward extreme
        // tracking. This is geometry-driven (not pixel-bin based), so small time shifts
        // don't change which points are selected → no jitter.
        const chTsMs = channelTs.getTime();
        wCtx.strokeStyle = WAVEFORM_COLOR;
        wCtx.lineWidth = 0.625;
        wCtx.lineJoin = 'round';

        // Transform visible samples to pixel coordinates
        const firstSampleTimeMs = chTsMs - ((samples.length - 1) / sampleRate) * 1000;
        const visStart = Math.max(0, Math.ceil((leftEdge - firstSampleTimeMs) / 1000 * sampleRate));
        const visEnd = Math.min(samples.length - 1, Math.floor((rightEdge - firstSampleTimeMs) / 1000 * sampleRate));
        const visCount = Math.max(0, visEnd - visStart + 1);

        if (visCount > 0) {
          const pxX = new Float64Array(visCount);
          const pxY = new Float64Array(visCount);
          for (let i = visStart; i <= visEnd; i++) {
            const sampleTime = chTsMs - ((samples.length - 1 - i) / sampleRate) * 1000;
            const j = i - visStart;
            pxX[j] = LEFT_MARGIN + ((sampleTime - leftEdge) / (windowSeconds * 1000)) * plotWidth;
            pxY[j] = mapY(samples[i] - mean);
          }

          wCtx.beginPath();
          drawSimplifiedPath(wCtx, pxX, pxY, visCount);
          wCtx.stroke();
        }

        // T010: Y-axis labels from nice-number ticks
        wCtx.fillStyle = FG_COLOR;
        wCtx.font = '10px Arial';
        wCtx.textAlign = 'right';
        const unitLabel = getUnitLabel(units, channelId);

        for (const tick of ticks) {
          const y = mapY(tick);
          wCtx.fillText(formatEngineering(tick), LEFT_MARGIN - 5, y + 3);
        }

        // Unit label (rotated)
        wCtx.save();
        wCtx.translate(12, waveformHeight / 2);
        wCtx.rotate(-Math.PI / 2);
        wCtx.textAlign = 'center';
        wCtx.font = '11px Arial';
        wCtx.fillText(unitLabel, 0, 0);
        wCtx.restore();

        // Draw alert markers on waveform
        drawAlertMarkers(wCtx, alerts, channelId, windowSeconds, LEFT_MARGIN, plotWidth, waveformHeight, rightEdge);
      }

      // Channel name legend
      wCtx.fillStyle = FG_COLOR;
      wCtx.font = 'bold 12px Arial';
      wCtx.textAlign = 'left';
      wCtx.fillText(channelId, LEFT_MARGIN + 5, 16);

      // T007: Waveform plot border
      wCtx.strokeStyle = BORDER_COLOR;
      wCtx.lineWidth = 1;
      wCtx.strokeRect(LEFT_MARGIN + 0.5, 0.5, plotWidth - 1, waveformHeight - 1);

      // Bandpass label (lower-left of waveform, only when filter enabled)
      if (settings.filter_waveform) {
        const bpLabel = `Bandpass (${settings.filter_highpass} - ${settings.filter_lowpass} Hz)`;
        wCtx.font = '9px Arial';
        const bpMetrics = wCtx.measureText(bpLabel);
        const bpX = LEFT_MARGIN + 5;
        const bpY = waveformHeight - 5;
        wCtx.fillStyle = 'rgba(32, 37, 48, 0.7)';
        wCtx.fillRect(bpX - 2, bpY - 9, bpMetrics.width + 4, 12);
        wCtx.fillStyle = FG_COLOR;
        wCtx.textAlign = 'left';
        wCtx.fillText(bpLabel, bpX, bpY);
      }

      // T005: Absolute time labels (HH:MM:SS UTC) below waveform, between plots
      {
        const ts = latestTimestampRef.current;
        if (ts) {
          const rightEdge = ts.getTime();
          const leftEdge = rightEdge - windowSeconds * 1000;
          const firstTick = Math.ceil(leftEdge / 10000) * 10000;

          wCtx.fillStyle = FG_COLOR;
          wCtx.font = '10px Arial';
          wCtx.textAlign = 'center';

          for (let t = firstTick; t <= rightEdge; t += 10000) {
            const x = LEFT_MARGIN + ((t - leftEdge) / (windowSeconds * 1000)) * plotWidth;
            const label = new Date(t).toISOString().substr(11, 8);
            wCtx.fillText(label, x, waveformHeight + 14);
          }
        }

        // T006: "Time (UTC)" title — below waveform when no spectrogram
        if (!showSpectrogram) {
          wCtx.fillStyle = FG_COLOR;
          wCtx.font = '10px Arial';
          wCtx.textAlign = 'center';
          wCtx.fillText('Time (UTC)', LEFT_MARGIN + plotWidth / 2, waveformHeight + TIME_AXIS_HEIGHT + TIME_LABEL_HEIGHT - 2);
        }
      }

      // --- Spectrogram (time-aligned with waveform) ---
      if (showSpectrogram) {
        const sCanvas = spectrogramCanvasRef.current;
        if (sCanvas) {
          const sCtx = sCanvas.getContext('2d');
          if (sCtx) {
            // Clear spectrogram canvas (same background as waveform)
            sCtx.fillStyle = BG_COLOR;
            sCtx.fillRect(0, 0, canvasWidth, spectrogramCanvasHeight);

            // Render spectrogram columns time-aligned with waveform
            const cols = spectrogramColumnsRef.current;
            const hd = hopDurationRef.current;
            const fBins = frequencyBinsRef.current;

            if (cols.length > 0 && hd > 0 && rightEdge > 0) {
              const firstColTs = spectrogramFirstColumnTimestampRef.current;
              const nyq = sampleRate / 2;
              const fMin = settings.spectrogram_freq_min || 0;
              const fMax = settings.spectrogram_freq_max || nyq;
              const logY = settings.spectrogram_log_y;
              const logMin = logY ? Math.log10(Math.max(fMin, 0.1)) : 0;
              const logMax = logY ? Math.log10(fMax) : 0;

              // Pre-compute pixel Y → FFT bin index lookup table
              const binLookup = new Uint16Array(spectrogramHeight);
              for (let y = 0; y < spectrogramHeight; y++) {
                const frac = (spectrogramHeight - 1 - y) / (spectrogramHeight - 1);
                let freq: number;
                if (logY) {
                  freq = Math.pow(10, logMin + frac * (logMax - logMin));
                } else {
                  freq = fMin + frac * (fMax - fMin);
                }
                const bin = Math.round((freq / nyq) * (fBins - 1));
                binLookup[y] = Math.max(0, Math.min(fBins - 1, bin));
              }

              // Column-driven rendering with timestamp-based positioning
              const imageData = sCtx.createImageData(plotWidth, spectrogramHeight);
              const pixels = imageData.data;

              // Skip columns before visible window
              const firstVisibleIdx = Math.max(0, Math.floor((leftEdge - firstColTs) / (hd * 1000)));

              for (let ci = firstVisibleIdx; ci < cols.length; ci++) {
                const colTime = firstColTs + ci * hd * 1000;
                if (colTime > rightEdge) break;

                const col = cols[ci];
                if (!col || col.length !== fBins) continue;

                // Compute pixel x range for this column
                const x = Math.round(((colTime - leftEdge) / (windowSeconds * 1000)) * plotWidth);
                const nextColTime = colTime + hd * 1000;
                const nextX = Math.round(((nextColTime - leftEdge) / (windowSeconds * 1000)) * plotWidth);
                const xStart = Math.max(0, x);
                const xEnd = Math.min(plotWidth, nextX);

                for (let px = xStart; px < xEnd; px++) {
                  for (let y = 0; y < spectrogramHeight; y++) {
                    const value = col[binLookup[y]];
                    const clampedIdx = Math.max(0, Math.min(INFERNO_RGB.length - 1, value));
                    const [r, g, b] = INFERNO_RGB[clampedIdx];
                    const pixelOffset = (y * plotWidth + px) * 4;
                    pixels[pixelOffset] = r;
                    pixels[pixelOffset + 1] = g;
                    pixels[pixelOffset + 2] = b;
                    pixels[pixelOffset + 3] = 255;
                  }
                }
              }

              sCtx.putImageData(imageData, LEFT_MARGIN, SPEC_TOP_PAD);
            }

            // Draw alert markers on spectrogram (aligned with waveform, offset by top pad)
            sCtx.save();
            sCtx.translate(0, SPEC_TOP_PAD);
            drawAlertMarkers(sCtx, alerts, channelId, windowSeconds, LEFT_MARGIN, plotWidth, spectrogramHeight, rightEdge);
            sCtx.restore();

            // Hz labels on left margin
            const nyquist = sampleRate / 2;
            const freqMin = settings.spectrogram_freq_min || 0;
            const freqMax = settings.spectrogram_freq_max || nyquist;

            sCtx.fillStyle = FG_COLOR;
            sCtx.font = '9px Arial';
            sCtx.textAlign = 'right';

            if (settings.spectrogram_log_y) {
              const logTicks = [0.5, 1, 2, 5, 10, 20, 50].filter(f => f >= freqMin && f <= freqMax);
              for (const freq of logTicks) {
                const logMin = Math.log10(Math.max(freqMin, 0.1));
                const logMax = Math.log10(freqMax);
                const logFreq = Math.log10(freq);
                const y = SPEC_TOP_PAD + spectrogramHeight - ((logFreq - logMin) / (logMax - logMin)) * spectrogramHeight;
                sCtx.fillText(`${freq}`, 25, y + 3);
              }
            } else {
              const freqTicks = 5;
              for (let i = 0; i <= freqTicks; i++) {
                const freq = freqMin + (freqMax - freqMin) * (i / freqTicks);
                const y = SPEC_TOP_PAD + spectrogramHeight - (i / freqTicks) * spectrogramHeight;
                sCtx.fillText(`${Math.round(freq)}`, 25, y + 3);
              }
            }

            // T015: "Frequency (Hz)" label (rotated)
            sCtx.save();
            sCtx.translate(8, SPEC_TOP_PAD + spectrogramHeight / 2);
            sCtx.rotate(-Math.PI / 2);
            sCtx.textAlign = 'center';
            sCtx.font = '9px Arial';
            sCtx.fillText('Frequency (Hz)', 0, 0);
            sCtx.restore();

            // T008: Spectrogram plot border
            sCtx.strokeStyle = BORDER_COLOR;
            sCtx.lineWidth = 1;
            sCtx.strokeRect(LEFT_MARGIN + 0.5, SPEC_TOP_PAD + 0.5, plotWidth - 1, spectrogramHeight - 1);

            // Range label (lower-left of spectrogram)
            {
              const rangeLabel = `Range (${settings.spectrogram_freq_min} - ${settings.spectrogram_freq_max} Hz)`;
              sCtx.font = '9px Arial';
              const rangeMetrics = sCtx.measureText(rangeLabel);
              const rX = LEFT_MARGIN + 5;
              const rY = SPEC_TOP_PAD + spectrogramHeight - 5;
              sCtx.fillStyle = 'rgba(32, 37, 48, 0.7)';
              sCtx.fillRect(rX - 2, rY - 9, rangeMetrics.width + 4, 12);
              sCtx.fillStyle = FG_COLOR;
              sCtx.textAlign = 'left';
              sCtx.fillText(rangeLabel, rX, rY);
            }

            // T006: "Time (UTC)" title below spectrogram
            sCtx.fillStyle = FG_COLOR;
            sCtx.font = '10px Arial';
            sCtx.textAlign = 'center';
            sCtx.fillText('Time (UTC)', LEFT_MARGIN + plotWidth / 2, SPEC_TOP_PAD + spectrogramHeight + TIME_LABEL_HEIGHT - 2);
          }
        }
      }
    };

    // Use requestAnimationFrame for synchronized rendering across all channels.
    // All RAF callbacks fire in the same browser paint frame, ensuring channels
    // read the same latestTimestamp and scroll in perfect sync.
    let rafId: number;
    let lastRenderTime = 0;
    const renderLoop = (timestamp: number) => {
      // Throttle to ~30 FPS (33ms between frames)
      if (timestamp - lastRenderTime >= 33) {
        lastRenderTime = timestamp;
        render();
      }
      rafId = requestAnimationFrame(renderLoop);
    };
    rafId = requestAnimationFrame(renderLoop);
    return () => cancelAnimationFrame(rafId);
  }, [buffer, canvasWidth, waveformHeight, waveformCanvasHeight, windowSeconds, sampleRate, autoScale, alerts, channelId, showSpectrogram, spectrogramHeight, spectrogramCanvasHeight, settings, isBottomChannel, units]);

  return (
    <div ref={containerRef} className="relative">
      <canvas
        ref={waveformCanvasRef}
        width={canvasWidth}
        height={waveformCanvasHeight}
        className="block"
        style={{ width: '100%', background: BG_COLOR }}
      />
      {showSpectrogram && (
        <canvas
          ref={spectrogramCanvasRef}
          width={canvasWidth}
          height={spectrogramCanvasHeight}
          className="block"
          style={{ width: '100%', background: BG_COLOR }}
        />
      )}
    </div>
  );
};

function drawAlertMarkers(
  ctx: CanvasRenderingContext2D,
  alerts: VisualAlertMarker[],
  channelId: string,
  windowSeconds: number,
  xOffset: number,
  plotWidth: number,
  height: number,
  rightEdge: number,
) {
  if (rightEdge <= 0) return;
  const leftEdge = rightEdge - windowSeconds * 1000;
  for (const alert of alerts) {
    if (alert.channel !== channelId) continue;

    const alertMs = new Date(alert.timestamp).getTime();

    if (alertMs >= leftEdge && alertMs <= rightEdge) {
      const x = xOffset + ((alertMs - leftEdge) / (windowSeconds * 1000)) * plotWidth;

      ctx.beginPath();
      ctx.setLineDash([5, 5]);
      ctx.strokeStyle = alert.type === 'Alarm' ? TRIGGER_COLOR : RESET_COLOR;
      ctx.lineWidth = 2;
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
      ctx.setLineDash([]);
    }
  }
}

export default ChannelPairCanvas;
