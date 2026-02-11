'use client';

import { useWebSocket } from '../../hooks/useWebSocket';
import { useAlerts } from '../../hooks/useAlerts';
import { useEffect, useRef, useState, useCallback } from 'react';
import { RingBuffer } from '../../lib/RingBuffer';
import ChannelPairCanvas from '../../components/ChannelPairCanvas';
import ControlPanel from '../../components/ControlPanel';
import AlertSettingsPanel from '../../components/AlertSettingsPanel';
import PerformanceMonitor from '../../components/PerformanceMonitor';
import IntensityBadge from '../../components/IntensityBadge';
import { PlotSettings, VisualAlertMarker, IntensityIndicatorState } from '../../lib/types';
import { getBackendOrigin, getWsUrl } from '../../lib/api';

const DEFAULT_SETTINGS: PlotSettings = {
  active_channels: [],
  window_seconds: 90,
  auto_scale: true,
  theme: 'dark',
  save_pct: 0.7,
  show_spectrogram: true,
  spectrogram_freq_min: 0,
  spectrogram_freq_max: 50,
  spectrogram_log_y: false,
};

function channelSortKey(ch: string): [number, string] {
  if (ch.endsWith('Z')) return [0, ch];
  if (ch.endsWith('E')) return [1, ch];
  if (ch.endsWith('N')) return [2, ch];
  return [3, ch];
}

interface SpectrogramState {
  columns: Uint8Array[];
  frequencyBins: number;
  sampleRate: number;
  hopDuration: number;
  totalReceived: number;
}

export default function Home() {
  const [settings, setSettings] = useState<PlotSettings>(DEFAULT_SETTINGS);
  const [availableChannels, setAvailableChannels] = useState<string[]>([]);
  const [buffers, setBuffers] = useState<Record<string, RingBuffer>>({});
  const [visualAlerts, setVisualAlerts] = useState<VisualAlertMarker[]>([]);
  const [spectrogramData, setSpectrogramData] = useState<Record<string, SpectrogramState>>({});
  const [eventCount, setEventCount] = useState(0);
  const [stationName, setStationName] = useState('');
  const [intensityState, setIntensityState] = useState<IntensityIndicatorState>({
    visible: false,
    maxIntensity: 0,
    maxClass: '0',
    triggerTime: null,
    resetTime: null,
    fadeoutTimer: null,
  });

  const [channelTimestamps, setChannelTimestamps] = useState<Record<string, Date>>({});

  const buffersRef = useRef<Record<string, RingBuffer>>({});
  const spectrogramRef = useRef<Record<string, SpectrogramState>>({});
  const channelTimestampsRef = useRef<Record<string, Date>>({});
  const fadeoutTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleSpectrogramData = useCallback((
    channelId: string,
    columns: Uint8Array[],
    frequencyBins: number,
    sampleRate: number,
    hopDuration: number,
  ) => {
    const prev = spectrogramRef.current[channelId];
    const existingCols = prev?.columns || [];
    const updatedCols = [...existingCols, ...columns];

    // Keep enough columns for max window (300s at ~7.7 cols/sec ≈ 2300)
    const maxCols = 3000;
    const trimmed = updatedCols.length > maxCols
      ? updatedCols.slice(updatedCols.length - maxCols)
      : updatedCols;

    spectrogramRef.current[channelId] = {
      columns: trimmed,
      frequencyBins,
      sampleRate,
      hopDuration,
      totalReceived: (prev?.totalReceived || 0) + columns.length,
    };
    setSpectrogramData({ ...spectrogramRef.current });
  }, []);

  const wsUrl = getWsUrl();
  const { isConnected, lastMessage } = useWebSocket(wsUrl, {
    onSpectrogramData: handleSpectrogramData,
  });
  useAlerts(lastMessage); // side-effect: audio playback on alert

  useEffect(() => {
    const api = getBackendOrigin();
    fetch(`${api}/api/settings`)
      .then(res => res.json())
      .then(data => {
        setSettings(prev => ({ ...prev, ...data }));
      })
      .catch(err => console.error('Failed to fetch settings:', err));

    fetch(`${api}/api/channels`)
      .then(res => res.json())
      .then((data: string[]) => {
        setAvailableChannels(data);
        // Auto-populate active_channels if empty (first load)
        setSettings(prev => {
          if (prev.active_channels.length === 0 && data.length > 0) {
            return { ...prev, active_channels: data };
          }
          return prev;
        });
      })
      .catch(err => console.error('Failed to fetch channels:', err));

    fetch(`${api}/api/station`)
      .then(res => res.json())
      .then((name: string) => { if (name) setStationName(name); })
      .catch(err => console.error('Failed to fetch station name:', err));
  }, []);

  const handleSettingsChange = (newSettings: PlotSettings) => {
    setSettings(newSettings);
    fetch(`${getBackendOrigin()}/api/settings`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newSettings),
    }).catch(err => console.error('Failed to save settings:', err));
  };

  useEffect(() => {
    if (!lastMessage) return;

    if (lastMessage.type === 'Waveform') {
      const { channel_id, timestamp, samples, sample_rate } = lastMessage.data;
      if (!settings.active_channels.includes(channel_id)) return;

      if (!buffersRef.current[channel_id]) {
        buffersRef.current[channel_id] = new RingBuffer(300 * sample_rate);
        setBuffers({ ...buffersRef.current });
      }
      buffersRef.current[channel_id].pushMany(samples);

      // Track latest timestamp for absolute time axis
      const latestTs = new Date(Date.parse(timestamp) + (samples.length / sample_rate) * 1000);
      channelTimestampsRef.current[channel_id] = latestTs;
      setChannelTimestamps({ ...channelTimestampsRef.current });
    } else if (lastMessage.type === 'AlertStart') {
      const { channel, timestamp } = lastMessage.data;
      setVisualAlerts(prev => [...prev, { type: 'Alarm' as const, channel, timestamp }].slice(-50));
      setEventCount(prev => prev + 1);

      // Intensity badge: activate
      if (fadeoutTimerRef.current) {
        clearTimeout(fadeoutTimerRef.current);
        fadeoutTimerRef.current = null;
      }
      setIntensityState({
        visible: true,
        maxIntensity: 0,
        maxClass: '0',
        triggerTime: new Date(timestamp),
        resetTime: null,
        fadeoutTimer: null,
      });
    } else if (lastMessage.type === 'AlertEnd') {
      const { channel, timestamp } = lastMessage.data;
      setVisualAlerts(prev => [...prev, { type: 'Reset' as const, channel, timestamp }].slice(-50));

      // Intensity badge: start 30s fadeout
      setIntensityState(prev => {
        if (!prev.visible) return prev;
        const timer = setTimeout(() => {
          setIntensityState(s => ({ ...s, visible: false, fadeoutTimer: null }));
          fadeoutTimerRef.current = null;
        }, 30000);
        fadeoutTimerRef.current = timer;
        return { ...prev, resetTime: new Date(timestamp), fadeoutTimer: timer };
      });
    } else if (lastMessage.type === 'Intensity') {
      const { instrumental_intensity, intensity_class } = lastMessage.data;
      setIntensityState(prev => {
        if (!prev.visible) return prev;
        if (instrumental_intensity > prev.maxIntensity) {
          return { ...prev, maxIntensity: instrumental_intensity, maxClass: intensity_class };
        }
        return prev;
      });
    }
  }, [lastMessage, settings.active_channels]);

  // Sort channels: Z first, E second, N third, others last
  const sortedChannels = [...settings.active_channels].sort((a, b) => {
    const [aKey, aName] = channelSortKey(a);
    const [bKey, bName] = channelSortKey(b);
    if (aKey !== bKey) return aKey - bKey;
    return aName.localeCompare(bName);
  });

  return (
    <main className="min-h-screen p-4 md:p-8 bg-[#202530]">
      <PerformanceMonitor />

      {/* Intensity Badge (top-right) */}
      {intensityState.visible && (
        <IntensityBadge
          maxClass={intensityState.maxClass}
        />
      )}

      <div className="max-w-7xl mx-auto flex flex-col lg:flex-row gap-8">
        <div className="flex-1 space-y-2">
          {/* Header */}
          <header className="relative mb-4">
            {/* rsudp-rust branding */}
            <span className="absolute left-0 top-0 text-xs text-gray-400 font-mono">rsudp-rust</span>

            {/* Title */}
            <h1 className="text-center text-lg font-semibold text-gray-300">
              {stationName || 'Station'} Live Data - Detected Events: {eventCount}
            </h1>

            {/* Connection status */}
            <div className="absolute right-0 top-0 flex items-center gap-2">
              <span className={`h-2 w-2 rounded-full ${isConnected ? 'bg-emerald-500' : 'bg-rose-500 animate-pulse'}`}></span>
              <span className="text-xs font-bold text-gray-500 uppercase tracking-wider">
                {isConnected ? 'Live' : 'Offline'}
              </span>
            </div>
          </header>

          {/* Channel Pairs */}
          <div className="space-y-1">
            {sortedChannels.length === 0 ? (
              <div className="h-96 flex flex-col items-center justify-center border-2 border-dashed border-gray-600 rounded text-gray-500">
                <p className="font-medium text-lg">No active channels</p>
              </div>
            ) : (
              sortedChannels.map((id, idx) => {
                const specState = spectrogramData[id];
                return (
                  <ChannelPairCanvas
                    key={id}
                    channelId={id}
                    buffer={buffers[id] || new RingBuffer(100)}
                    spectrogramColumns={specState?.columns || []}
                    frequencyBins={specState?.frequencyBins || 65}
                    sampleRate={specState?.sampleRate || 100}
                    hopDuration={specState?.hopDuration || 0.13}
                    windowSeconds={settings.window_seconds}
                    autoScale={settings.auto_scale}
                    alerts={visualAlerts}
                    settings={settings}
                    isBottomChannel={idx === sortedChannels.length - 1}
                    units="CHAN"
                    latestTimestamp={channelTimestamps[id] || null}
                  />
                );
              })
            )}
          </div>
        </div>

        <aside className="lg:w-80 space-y-6">
          <ControlPanel
            settings={settings}
            onSettingsChange={handleSettingsChange}
            availableChannels={availableChannels}
          />
          <AlertSettingsPanel />
        </aside>
      </div>
    </main>
  );
}
