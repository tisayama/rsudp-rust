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
}

export type AlertEventType = 'Alarm' | 'Reset';

export interface VisualAlertMarker {
  type: 'Alarm' | 'Reset';
  timestamp: string;
  channel: string;
}

export interface AlertEvent {
  id: string;
  channel: string;
  trigger_time: string;
  reset_time: string | null;
  max_ratio: number;
  snapshot_path: string | null;
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
}

export type WsMessage = 
  | { type: 'Waveform', data: WaveformPacket }
  | { type: 'Alert', data: { timestamp: string, channel: string, message: string } }
  | { type: 'Intensity', data: IntensityResult }
  | { type: 'AlertStart', data: { id: string, channel: string, timestamp: string } }
  | { type: 'AlertEnd', data: { id: string, channel: string, timestamp: string, max_ratio: number } };
