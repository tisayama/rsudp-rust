'use client';

import React, { useEffect, useRef } from 'react';
import { RingBuffer } from '../lib/RingBuffer';
import { VisualAlertMarker, PlotSettings } from '../lib/types';
import { INFERNO_RGB } from '../lib/inferno-colormap';
import { formatEngineering } from '../lib/engineering-format';
import { computeNiceTicks } from '../lib/nice-number';

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

  const showSpectrogram = settings.show_spectrogram;
  const waveformHeight = 200;
  const spectrogramHeight = showSpectrogram ? 100 : 0;
  const canvasWidth = 900;

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

      // --- Waveform ---
      wCtx.fillStyle = BG_COLOR;
      wCtx.fillRect(0, 0, canvasWidth, waveformCanvasHeight);

      if (samples.length > 0) {
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

        // Draw waveform line
        wCtx.strokeStyle = WAVEFORM_COLOR;
        wCtx.lineWidth = 0.45;
        wCtx.lineJoin = 'round';
        wCtx.beginPath();
        for (let i = 0; i < samples.length; i++) {
          const x = LEFT_MARGIN + (i / (windowSeconds * sampleRate)) * plotWidth;
          const y = mapY(samples[i] - mean);
          if (i === 0) wCtx.moveTo(x, y);
          else wCtx.lineTo(x, y);
        }
        wCtx.stroke();

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
        drawAlertMarkers(wCtx, alerts, channelId, windowSeconds, LEFT_MARGIN, plotWidth, waveformHeight);
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

            if (cols.length > 0 && hd > 0) {
              // Number of columns that cover the visible time window
              const maxVisibleCols = Math.ceil(windowSeconds / hd);
              const startCol = Math.max(0, cols.length - maxVisibleCols);
              const visibleCols = cols.slice(startCol);
              const numCols = visibleCols.length;

              // Render as ImageData aligned with waveform plot area
              const imageData = sCtx.createImageData(plotWidth, spectrogramHeight);
              const pixels = imageData.data;

              for (let x = 0; x < plotWidth; x++) {
                // Map pixel x to column index (same time mapping as waveform)
                const colIdx = Math.floor((x / plotWidth) * maxVisibleCols);
                if (colIdx >= numCols) continue;

                const col = visibleCols[colIdx];
                if (!col || col.length !== fBins) continue;

                for (let y = 0; y < spectrogramHeight; y++) {
                  // Map canvas row to frequency bin (bottom = 0 Hz, top = Nyquist)
                  const binIndex = Math.floor(((spectrogramHeight - 1 - y) / (spectrogramHeight - 1)) * (fBins - 1));
                  const value = col[Math.min(binIndex, fBins - 1)];
                  const clampedIdx = Math.max(0, Math.min(INFERNO_RGB.length - 1, value));
                  const [r, g, b] = INFERNO_RGB[clampedIdx];
                  const pixelOffset = (y * plotWidth + x) * 4;
                  pixels[pixelOffset] = r;
                  pixels[pixelOffset + 1] = g;
                  pixels[pixelOffset + 2] = b;
                  pixels[pixelOffset + 3] = 255;
                }
              }

              sCtx.putImageData(imageData, LEFT_MARGIN, SPEC_TOP_PAD);
            }

            // Draw alert markers on spectrogram (aligned with waveform, offset by top pad)
            sCtx.save();
            sCtx.translate(0, SPEC_TOP_PAD);
            drawAlertMarkers(sCtx, alerts, channelId, windowSeconds, LEFT_MARGIN, plotWidth, spectrogramHeight);
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

            // T006: "Time (UTC)" title below spectrogram
            sCtx.fillStyle = FG_COLOR;
            sCtx.font = '10px Arial';
            sCtx.textAlign = 'center';
            sCtx.fillText('Time (UTC)', LEFT_MARGIN + plotWidth / 2, SPEC_TOP_PAD + spectrogramHeight + TIME_LABEL_HEIGHT - 2);
          }
        }
      }
    };

    const interval = setInterval(render, 33); // ~30 FPS
    return () => clearInterval(interval);
  }, [buffer, canvasWidth, waveformHeight, waveformCanvasHeight, windowSeconds, sampleRate, autoScale, alerts, channelId, showSpectrogram, spectrogramHeight, spectrogramCanvasHeight, settings, isBottomChannel, units]);

  return (
    <div className="relative">
      <canvas
        ref={waveformCanvasRef}
        width={canvasWidth}
        height={waveformCanvasHeight}
        className="w-full h-auto block"
        style={{ background: BG_COLOR }}
      />
      {showSpectrogram && (
        <canvas
          ref={spectrogramCanvasRef}
          width={canvasWidth}
          height={spectrogramCanvasHeight}
          className="w-full h-auto block"
          style={{ background: BG_COLOR }}
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
) {
  const now = new Date();
  for (const alert of alerts) {
    if (alert.channel !== channelId) continue;

    const alertTime = new Date(alert.timestamp);
    const diffMs = now.getTime() - alertTime.getTime();
    const diffSec = diffMs / 1000;

    if (diffSec >= 0 && diffSec <= windowSeconds) {
      const x = xOffset + plotWidth - (diffSec / windowSeconds) * plotWidth;

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
