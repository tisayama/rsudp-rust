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

export interface AlertEvent {
  event_type: AlertEventType;
  timestamp: string;
  channel_id: string;
  max_ratio?: number;
}

export interface IntensityResult {
  instrumental_intensity: number;
  intensity_class: string;
  timestamp: string;
}

export type WsMessage = 
  | { type: 'Waveform', data: WaveformPacket }
  | { type: 'Alert', data: AlertEvent }
  | { type: 'Intensity', data: IntensityResult };
