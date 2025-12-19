'use client';

import { useWebSocket } from '../../hooks/useWebSocket';
import { useEffect, useRef, useState } from 'react';
import { RingBuffer } from '../../lib/RingBuffer';
import WaveformCanvas from '../../components/WaveformCanvas';
import ControlPanel from '../../components/ControlPanel';
import PerformanceMonitor from '../../components/PerformanceMonitor';
import { AlertEvent, PlotSettings } from '../../lib/types';

const DEFAULT_SETTINGS: PlotSettings = {
  active_channels: ['SHZ', 'EHZ'],
  window_seconds: 60,
  auto_scale: true,
  theme: 'light',
};

export default function Home() {
  const { isConnected, lastMessage } = useWebSocket('ws://localhost:8080/ws');
  const [settings, setSettings] = useState<PlotSettings>(DEFAULT_SETTINGS);
  const [availableChannels, setAvailableChannels] = useState<string[]>([]);
  const [buffers, setBuffers] = useState<Record<string, RingBuffer>>({});
  const [alerts, setAlerts] = useState<AlertEvent[]>([]);
  
  const buffersRef = useRef<Record<string, RingBuffer>>({});

  useEffect(() => {
    fetch('http://localhost:8080/api/settings')
      .then(res => res.json())
      .then(data => setSettings(data))
      .catch(err => console.error('Failed to fetch settings:', err));

    fetch('http://localhost:8080/api/channels')
      .then(res => res.json())
      .then(data => setAvailableChannels(data))
      .catch(err => console.error('Failed to fetch channels:', err));
  }, []);

  const handleSettingsChange = (newSettings: PlotSettings) => {
    setSettings(newSettings);
    fetch('http://localhost:8080/api/settings', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newSettings),
    }).catch(err => console.error('Failed to save settings:', err));
  };

  useEffect(() => {
    if (!lastMessage) return;

    if (lastMessage.type === 'Waveform') {
      const { channel_id, samples, sample_rate } = lastMessage.data;
      if (!settings.active_channels.includes(channel_id)) return;

      if (!buffersRef.current[channel_id]) {
        buffersRef.current[channel_id] = new RingBuffer(300 * sample_rate);
        setBuffers({ ...buffersRef.current });
      }
      buffersRef.current[channel_id].pushMany(samples);
    } else if (lastMessage.type === 'Alert') {
      setAlerts(prev => [...prev, lastMessage.data].slice(-20)); // Keep last 20
    }
  }, [lastMessage, settings.active_channels]);

  return (
    <main className="min-h-screen bg-slate-50 p-4 md:p-8">
      <PerformanceMonitor />
      <div className="max-w-7xl mx-auto flex flex-col lg:flex-row gap-8">
        <div className="flex-1 space-y-6">
          <header className="flex justify-between items-center mb-2">
            <h1 className="text-3xl font-black text-slate-900 tracking-tight italic">RSRUST<span className="text-blue-600">UDP</span> MONITOR</h1>
            <div className="flex items-center gap-3 px-4 py-2 bg-white rounded-full shadow-sm border border-slate-200">
              <span className={`h-2.5 w-2.5 rounded-full ${isConnected ? 'bg-emerald-500' : 'bg-rose-500 animate-pulse'}`}></span>
              <span className="text-xs font-bold text-slate-600 uppercase tracking-wider">
                {isConnected ? 'Live' : 'Offline'}
              </span>
            </div>
          </header>

          <div className="grid gap-4">
            {settings.active_channels.length === 0 ? (
              <div className="h-96 flex flex-col items-center justify-center bg-white border-2 border-dashed border-slate-300 rounded-2xl text-slate-400">
                <p className="font-medium text-lg">No active channels</p>
              </div>
            ) : (
              settings.active_channels.map(id => (
                <div key={id} className="relative group">
                  <WaveformCanvas 
                    buffer={buffers[id] || new RingBuffer(100)} 
                    channelId={id} 
                    width={900} 
                    height={220}
                    windowSeconds={settings.window_seconds}
                    sampleRate={100}
                    autoScale={settings.auto_scale}
                    alerts={alerts}
                  />
                </div>
              ))
            )}
          </div>
        </div>

        <aside className="lg:w-80 space-y-6">
          <ControlPanel 
            settings={settings} 
            onSettingsChange={handleSettingsChange}
            availableChannels={availableChannels}
          />
        </aside>
      </div>
    </main>
  );
}