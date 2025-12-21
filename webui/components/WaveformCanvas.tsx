'use client';

import React, { useEffect, useRef } from 'react';
import { RingBuffer } from '../lib/RingBuffer';
import { VisualAlertMarker } from '../lib/types';

interface WaveformCanvasProps {
  buffer: RingBuffer;
  channelId: string;
  width: number;
  height: number;
  windowSeconds: number;
  sampleRate: number;
  autoScale: boolean;
  alerts: VisualAlertMarker[];
}

const WaveformCanvas: React.FC<WaveformCanvasProps> = ({ 
  buffer, 
  channelId, 
  width, 
  height, 
  windowSeconds, 
  sampleRate,
  autoScale,
  alerts
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const render = () => {
      const samples = buffer.getTail(windowSeconds * sampleRate);
      
      // Clear
      ctx.clearRect(0, 0, width, height);

      if (samples.length === 0) return;

      // Draw Grid
      ctx.strokeStyle = '#e2e8f0';
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let i = 1; i < 4; i++) {
        const y = (height / 4) * i;
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
      }
      ctx.stroke();

      // Find Scale
      let yMin = -1000;
      let yMax = 1000;
      
      if (autoScale) {
        let maxAbs = 0;
        for (const s of samples) {
          if (Math.abs(s) > maxAbs) maxAbs = Math.abs(s);
        }
        if (maxAbs > 0) {
          yMin = -maxAbs * 1.1;
          yMax = maxAbs * 1.1;
        }
      }

      const mapY = (val: number) => {
        return height - ((val - yMin) / (yMax - yMin)) * height;
      };

      // Draw Waveform
      ctx.strokeStyle = '#2563eb';
      ctx.lineWidth = 1.5;
      ctx.lineJoin = 'round';
      ctx.beginPath();
      for (let i = 0; i < samples.length; i++) {
        const x = (i / (windowSeconds * sampleRate)) * width;
        const y = mapY(samples[i]);
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Draw Alert Markers
      const now = new Date();
      alerts.forEach(alert => {
        if (alert.channel !== channelId) return;
        
        const alertTime = new Date(alert.timestamp);
        const diffMs = now.getTime() - alertTime.getTime();
        const diffSec = diffMs / 1000;

        if (diffSec >= 0 && diffSec <= windowSeconds) {
          const x = width - (diffSec / windowSeconds) * width;
          
          ctx.beginPath();
          ctx.setLineDash([5, 5]);
          ctx.strokeStyle = alert.type === 'Alarm' ? '#ef4444' : '#10b981';
          ctx.lineWidth = 2;
          ctx.moveTo(x, 0);
          ctx.lineTo(x, height);
          ctx.stroke();
          ctx.setLineDash([]);

          ctx.fillStyle = alert.type === 'Alarm' ? '#ef4444' : '#10b981';
          ctx.font = 'bold 10px sans-serif';
          ctx.fillText(alert.type.toUpperCase(), x + 5, 15);
        }
      });
    };

    const interval = setInterval(render, 30);
    return () => clearInterval(interval);
  }, [buffer, width, height, windowSeconds, sampleRate, autoScale, alerts, channelId]);

  return (
    <div className="bg-white rounded-2xl shadow-sm border border-slate-200 overflow-hidden">
      <div className="px-4 py-2 bg-slate-50 border-b border-slate-200 flex justify-between items-center">
        <span className="font-bold text-slate-700 text-sm tracking-widest">{channelId}</span>
        <span className="text-[10px] font-black text-slate-400 uppercase tracking-tighter">Live Waveform</span>
      </div>
      <canvas 
        ref={canvasRef} 
        width={width} 
        height={height}
        className="w-full h-auto block"
      />
    </div>
  );
};

export default WaveformCanvas;