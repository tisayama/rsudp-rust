'use client';

import React, { useEffect, useRef } from 'react';
import { RingBuffer } from '../lib/RingBuffer';
import { AlertEvent } from '../lib/types';

interface WaveformCanvasProps {
  buffer: RingBuffer;
  channelId: string;
  width?: number;
  height?: number;
  windowSeconds: number;
  sampleRate: number;
  autoScale: boolean;
  alerts: AlertEvent[];
}

const WaveformCanvas: React.FC<WaveformCanvasProps> = ({ 
  buffer, 
  channelId, 
  width = 800, 
  height = 200,
  windowSeconds,
  sampleRate,
  autoScale,
  alerts
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    let animationId: number;

    const render = () => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      const dpr = window.devicePixelRatio || 1;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx.scale(dpr, dpr);

      ctx.clearRect(0, 0, width, height);
      
      const totalSamples = windowSeconds * sampleRate;
      const currentLen = buffer.length;
      if (currentLen < 2) {
        animationId = requestAnimationFrame(render);
        return;
      }

      const viewLen = Math.min(currentLen, totalSamples);
      const startIndex = currentLen - viewLen;
      const step = width / (totalSamples - 1);

      // Auto-scale
      let scale = 1;
      if (autoScale) {
        let maxAbs = 0;
        for (let i = 0; i < viewLen; i++) {
          const val = Math.abs(buffer.get(startIndex + i));
          if (val > maxAbs) maxAbs = val;
        }
        scale = maxAbs === 0 ? 1 : (height / 2.2) / maxAbs;
      } else {
        scale = 0.001;
      }

      // Draw Waveform
      ctx.beginPath();
      ctx.strokeStyle = '#3b82f6';
      ctx.lineWidth = 1.5;
      for (let i = 0; i < viewLen; i++) {
        const x = (totalSamples - viewLen + i) * step;
        const y = (height / 2) - (buffer.get(startIndex + i) * scale);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Draw Alerts
      const now = new Date();
      alerts.forEach(alert => {
        if (alert.channel_id !== channelId) return;
        
        const alertTime = new Date(alert.timestamp);
        const diffMs = now.getTime() - alertTime.getTime();
        const diffSec = diffMs / 1000;

        if (diffSec < windowSeconds && diffSec >= 0) {
          const x = width - (diffSec * sampleRate * step);
          
          ctx.beginPath();
          ctx.setLineDash([5, 5]);
          ctx.strokeStyle = alert.event_type === 'Alarm' ? '#ef4444' : '#10b981';
          ctx.lineWidth = 2;
          ctx.moveTo(x, 0);
          ctx.lineTo(x, height);
          ctx.stroke();
          ctx.setLineDash([]);

          ctx.fillStyle = alert.event_type === 'Alarm' ? '#ef4444' : '#10b981';
          ctx.font = 'bold 10px sans-serif';
          ctx.fillText(alert.event_type.toUpperCase(), x + 5, 15);
        }
      });

      // UI Chrome
      ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
      ctx.fillRect(5, 5, 60, 20);
      ctx.fillStyle = '#1f2937';
      ctx.font = 'bold 12px sans-serif';
      ctx.fillText(channelId, 10, 20);

      animationId = requestAnimationFrame(render);
    };

    render();

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, [buffer, width, height, channelId, windowSeconds, sampleRate, autoScale, alerts]);

  return (
    <canvas
      ref={canvasRef}
      style={{ width: `${width}px`, height: `${height}px` }}
      className="rounded-lg bg-gray-50 border border-gray-200 shadow-inner"
    />
  );
};

export default WaveformCanvas;
