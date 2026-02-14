export interface WaveformPacket {
  channel_id: string;
  timestamp: string; // ISO 8601
  samples: number[];
  sample_rate: number;
}

export interface PlotSettings {
  active_channels: string[];
  window_seconds: number;
  auto_scale: boolean;
  theme: 'dark' | 'light';
  save_pct: number;
  show_spectrogram: boolean;
  spectrogram_freq_min: number;
  spectrogram_freq_max: number;
  spectrogram_log_y: boolean;
  deconvolve: boolean;
  filter_waveform: boolean;
  filter_highpass: number;
  filter_lowpass: number;
  filter_corners: number;
}

export type AlertEventType = 'Alarm' | 'Reset';

export interface VisualAlertMarker {
  type: 'Alarm' | 'Reset';
  timestamp: string;
  channel: string;
}

export interface IntensityResult {
  instrumental_intensity: number;
  intensity_class: string;
  timestamp: string;
}

export interface AlertSettings {
  audio_enabled: boolean;
  email_enabled: boolean;
  flash_enabled: boolean;
  smtp_host: string;
  smtp_port: number;
  smtp_user: string;
  smtp_pass: string;
  email_recipient: string;
  save_pct: number;
}

export interface SpectrogramColumn {
  data: Float32Array;
  timestamp: number; // microseconds since epoch
  frequencyBins: number;
}

export interface SpectrogramPacket {
  channelId: string;
  timestamp: number; // microseconds since epoch
  sampleRate: number;
  frequencyBins: number;
  columnsCount: number;
  columns: Float32Array[];
}

export interface IntensityIndicatorState {
  visible: boolean;
  maxIntensity: number;
  maxClass: string;
  triggerTime: Date | null;
  resetTime: Date | null;
  fadeoutTimer: ReturnType<typeof setTimeout> | null;
}

export type WsMessage =
  | { type: 'Waveform', data: WaveformPacket }
  | { type: 'Alert', data: { timestamp: string, channel: string, message: string } }
  | { type: 'Intensity', data: IntensityResult }
  | { type: 'AlertStart', data: { id: string, channel: string, timestamp: string } }
  | { type: 'AlertEnd', data: { id: string, channel: string, timestamp: string, max_ratio: number, message: string } }
  | { type: 'BackfillComplete', data: { channels: string[] } };
