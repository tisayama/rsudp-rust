'use client';

import React, { useEffect, useState, useRef, Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import { RingBuffer } from '../../../lib/RingBuffer';
import ChannelPairCanvas from '../../../components/ChannelPairCanvas';
import { PlotSettings, VisualAlertMarker } from '../../../lib/types';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface CaptureChannelData {
  samples: number[];
  start_time: string;
}

interface CaptureSpectrogramData {
  columns: number[][];
  frequency_bins: number;
  hop_duration: number;
  first_column_timestamp: number;
}

interface CaptureSettings {
  filter_waveform: boolean;
  filter_highpass: number;
  filter_lowpass: number;
  filter_corners: number;
  deconvolve: boolean;
  spectrogram_freq_min: number;
  spectrogram_freq_max: number;
  spectrogram_log_y: boolean;
}

interface CaptureDataResponse {
  station: string;
  sample_rate: number;
  channels: Record<string, CaptureChannelData>;
  spectrogram: Record<string, CaptureSpectrogramData>;
  sensitivity: Record<string, number>;
  settings: CaptureSettings;
}

// ---------------------------------------------------------------------------
// JMA Intensity Badge (inline, capture-specific)
// ---------------------------------------------------------------------------

const JMA_COLORS: Record<string, string> = {
  '0': '#F2F2FF',
  '1': '#F2F2FF',
  '2': '#00AAFF',
  '3': '#0041FF',
  '4': '#FAE696',
  '5-': '#FFE600',
  '5+': '#FF9900',
  '6-': '#FF2800',
  '6+': '#A50021',
  '7': '#B40068',
};

function getJMAColor(shindoClass: string): string {
  return JMA_COLORS[shindoClass] || '#F2F2FF';
}

function needsDarkText(shindoClass: string): boolean {
  return ['0', '1', '4', '5-'].includes(shindoClass);
}

interface CaptureIntensityBadgeProps {
  intensityClass: string;
  intensityValue: number;
}

const CaptureIntensityBadge: React.FC<CaptureIntensityBadgeProps> = ({
  intensityClass,
  intensityValue,
}) => {
  const bgColor = getJMAColor(intensityClass);
  const textColor = needsDarkText(intensityClass) ? '#333333' : '#FFFFFF';

  return (
    <div
      style={{
        backgroundColor: bgColor,
        width: 100,
        height: 100,
        borderRadius: 12,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        flexShrink: 0,
      }}
    >
      <span
        style={{
          color: textColor,
          fontSize: 40,
          fontWeight: 'bold',
          lineHeight: 1,
        }}
      >
        {intensityClass}
      </span>
      <span
        style={{
          color: textColor,
          fontSize: 14,
          opacity: 0.85,
          marginTop: 2,
        }}
      >
        {intensityValue.toFixed(1)}
      </span>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Channel sort: Z first, E second, N third, others last
// ---------------------------------------------------------------------------

function channelSortKey(ch: string): [number, string] {
  if (ch.endsWith('Z')) return [0, ch];
  if (ch.endsWith('E')) return [1, ch];
  if (ch.endsWith('N')) return [2, ch];
  return [3, ch];
}

// ---------------------------------------------------------------------------
// Capture page inner component (uses useSearchParams)
// ---------------------------------------------------------------------------

function CapturePageInner() {
  const searchParams = useSearchParams();

  const channels = searchParams.get('channels') || '';
  const start = searchParams.get('start') || '';
  const end = searchParams.get('end') || '';
  const intensityClass = searchParams.get('intensity_class') || '0';
  const intensityValue = parseFloat(searchParams.get('intensity_value') || '0');
  const backendUrl = searchParams.get('backend_url') || '';

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<CaptureDataResponse | null>(null);

  // Track how many canvases have completed their first render
  const channelListRef = useRef<string[]>([]);
  const renderedCountRef = useRef(0);

  // Fetch capture data from backend
  useEffect(() => {
    if (!channels || !start || !end || !backendUrl) {
      setError('Missing required query parameters: channels, start, end, backend_url');
      setLoading(false);
      return;
    }

    const url = `${backendUrl}/api/capture/data?channels=${encodeURIComponent(channels)}&start=${encodeURIComponent(start)}&end=${encodeURIComponent(end)}`;

    fetch(url)
      .then((res) => {
        if (!res.ok) {
          throw new Error(`Backend returned ${res.status}: ${res.statusText}`);
        }
        return res.json();
      })
      .then((json: CaptureDataResponse) => {
        setData(json);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message || 'Failed to fetch capture data');
        setLoading(false);
      });
  }, [channels, start, end, backendUrl]);

  // After data is loaded, wait for canvas renders and then signal completion
  useEffect(() => {
    if (!data) return;

    const channelIds = channels.split(',').filter((c) => c.length > 0);
    channelListRef.current = channelIds;
    renderedCountRef.current = 0;

    // ChannelPairCanvas uses requestAnimationFrame internally at ~30fps.
    // We wait for multiple animation frames to ensure all canvases have had
    // a chance to render their first frame, then set the captureReady signal.
    // 3 frames at 30fps ~= 100ms is a conservative wait.
    let frameCount = 0;
    const targetFrames = 5; // wait 5 rAF cycles to be safe

    const waitForRender = () => {
      frameCount++;
      if (frameCount >= targetFrames) {
        document.body.dataset.captureReady = 'true';
      } else {
        requestAnimationFrame(waitForRender);
      }
    };

    requestAnimationFrame(waitForRender);
  }, [data, channels]);

  // Error state
  if (error) {
    return (
      <div
        style={{
          width: 1000,
          padding: 40,
          backgroundColor: '#202530',
          color: '#ff6666',
          fontFamily: 'Arial, sans-serif',
        }}
      >
        <h2>Capture Error</h2>
        <p>{error}</p>
      </div>
    );
  }

  // Loading state
  if (loading || !data) {
    return (
      <div
        style={{
          width: 1000,
          padding: 40,
          backgroundColor: '#202530',
          color: '#cccccc',
          fontFamily: 'Arial, sans-serif',
        }}
      >
        Loading capture data...
      </div>
    );
  }

  // Parse channel list and sort
  const channelList = channels
    .split(',')
    .filter((c) => c.length > 0)
    .sort((a, b) => {
      const [aKey, aName] = channelSortKey(a);
      const [bKey, bName] = channelSortKey(b);
      if (aKey !== bKey) return aKey - bKey;
      return aName.localeCompare(bName);
    });

  // Compute time window in seconds
  const startMs = new Date(start).getTime();
  const endMs = new Date(end).getTime();
  const windowSeconds = Math.max(1, (endMs - startMs) / 1000);

  // Build PlotSettings from API response
  const plotSettings: PlotSettings = {
    active_channels: channelList,
    window_seconds: windowSeconds,
    auto_scale: true,
    theme: 'dark',
    save_pct: 0.7,
    show_spectrogram: true,
    spectrogram_freq_min: data.settings.spectrogram_freq_min,
    spectrogram_freq_max: data.settings.spectrogram_freq_max,
    spectrogram_log_y: data.settings.spectrogram_log_y,
    deconvolve: data.settings.deconvolve,
    filter_waveform: data.settings.filter_waveform,
    filter_highpass: data.settings.filter_highpass,
    filter_lowpass: data.settings.filter_lowpass,
    filter_corners: data.settings.filter_corners,
  };

  // Prepare per-channel data structures
  const emptyAlerts: VisualAlertMarker[] = [];

  return (
    <div
      style={{
        width: 1000,
        backgroundColor: '#202530',
        fontFamily: 'Arial, sans-serif',
        padding: '16px 0',
      }}
    >
      {/* Header: Station name + Intensity badge */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 20px 12px 20px',
        }}
      >
        <div>
          <div
            style={{
              color: '#cccccc',
              fontSize: 20,
              fontWeight: 'bold',
            }}
          >
            {data.station || 'Station'}
          </div>
          <div
            style={{
              color: '#999999',
              fontSize: 12,
              marginTop: 2,
            }}
          >
            {new Date(start).toISOString().replace('T', ' ').replace('Z', ' UTC')}
            {' \u2013 '}
            {new Date(end).toISOString().replace('T', ' ').replace('Z', ' UTC')}
          </div>
        </div>
        <CaptureIntensityBadge
          intensityClass={intensityClass}
          intensityValue={intensityValue}
        />
      </div>

      {/* Channel plots */}
      {channelList.map((channelId, idx) => {
        const chData = data.channels[channelId];
        const specData = data.spectrogram[channelId];

        // Create RingBuffer and populate with samples
        const sampleRate = data.sample_rate;
        const samples = chData?.samples || [];
        const bufferSize = Math.max(samples.length, sampleRate * windowSeconds);
        const buffer = new RingBuffer(Math.ceil(bufferSize));
        buffer.pushMany(samples);

        // Convert spectrogram columns from number[][] to Float32Array[]
        const spectrogramColumns: Float32Array[] = specData
          ? specData.columns.map((col) => new Float32Array(col))
          : [];

        const frequencyBins = specData?.frequency_bins || 65;
        const hopDuration = specData?.hop_duration || 0.13;
        const firstColumnTimestamp = specData?.first_column_timestamp || 0;

        // Compute channel latest timestamp from start_time + sample count
        const channelStartTime = chData?.start_time
          ? new Date(chData.start_time).getTime()
          : startMs;
        const channelLatestMs =
          channelStartTime + ((samples.length - 1) / sampleRate) * 1000;
        const channelLatestTimestamp = new Date(channelLatestMs);

        // Global latest timestamp = end of the requested window
        const latestTimestamp = new Date(endMs);

        return (
          <div
            key={channelId}
            style={{
              width: 1000,
              height: 500,
            }}
          >
            <ChannelPairCanvas
              channelId={channelId}
              buffer={buffer}
              spectrogramColumns={spectrogramColumns}
              frequencyBins={frequencyBins}
              sampleRate={sampleRate}
              hopDuration={hopDuration}
              windowSeconds={windowSeconds}
              autoScale={true}
              alerts={emptyAlerts}
              settings={plotSettings}
              isBottomChannel={idx === channelList.length - 1}
              units={plotSettings.deconvolve ? (data.settings.units || 'CHAN') : 'COUNTS'}
              latestTimestamp={latestTimestamp}
              channelLatestTimestamp={channelLatestTimestamp}
              spectrogramFirstColumnTimestamp={firstColumnTimestamp}
            />
          </div>
        );
      })}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Page wrapper with Suspense (required for useSearchParams)
// ---------------------------------------------------------------------------

export default function CapturePage() {
  return (
    <Suspense
      fallback={
        <div
          style={{
            width: 1000,
            padding: 40,
            backgroundColor: '#202530',
            color: '#cccccc',
            fontFamily: 'Arial, sans-serif',
          }}
        >
          Loading...
        </div>
      }
    >
      <CapturePageInner />
    </Suspense>
  );
}
