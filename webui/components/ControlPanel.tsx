'use client';

import React from 'react';
import { PlotSettings } from '../lib/types';

interface ControlPanelProps {
  settings: PlotSettings;
  onSettingsChange: (newSettings: PlotSettings) => void;
  availableChannels: string[];
}

const ControlPanel: React.FC<ControlPanelProps> = ({ settings, onSettingsChange, availableChannels }) => {
  const toggleChannel = (channel: string) => {
    const active = settings.active_channels.includes(channel);
    const next = active
      ? settings.active_channels.filter(c => c !== channel)
      : [...settings.active_channels, channel];
    onSettingsChange({ ...settings, active_channels: next });
  };

  return (
    <div className="bg-[#2a2f3d] p-6 rounded-xl border border-gray-700">
      <h2 className="text-xl font-bold mb-4 text-gray-300">Settings</h2>

      <div className="mb-6">
        <label className="block text-sm font-semibold text-gray-400 mb-2">Channels</label>
        <div className="flex flex-wrap gap-2">
          {availableChannels.map(channel => (
            <button
              key={channel}
              onClick={() => toggleChannel(channel)}
              className={`px-3 py-1.5 rounded-full text-xs font-bold transition-colors ${
                settings.active_channels.includes(channel)
                  ? 'bg-blue-600 text-white shadow-md'
                  : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
              }`}
            >
              {channel}
            </button>
          ))}
        </div>
      </div>

      <div className="mb-6">
        <label className="block text-sm font-semibold text-gray-400 mb-2">
          Time Window: {settings.window_seconds}s
        </label>
        <input
          type="range"
          min="5"
          max="300"
          step="5"
          value={settings.window_seconds}
          onChange={(e) => onSettingsChange({ ...settings, window_seconds: parseInt(e.target.value) })}
          className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-600"
        />
      </div>

      <div className="flex items-center justify-between mb-4">
        <label className="text-sm font-semibold text-gray-400">Auto-scale</label>
        <button
          onClick={() => onSettingsChange({ ...settings, auto_scale: !settings.auto_scale })}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
            settings.auto_scale ? 'bg-blue-600' : 'bg-gray-600'
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              settings.auto_scale ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>

      {/* Bandpass Filter Controls */}
      <div className="border-t border-gray-700 pt-4 mt-4">
        <h3 className="text-sm font-bold text-gray-300 mb-3">Bandpass Filter</h3>

        <div className="flex items-center justify-between mb-4">
          <label className="text-sm font-semibold text-gray-400">Filter Waveform</label>
          <button
            onClick={() => onSettingsChange({ ...settings, filter_waveform: !settings.filter_waveform })}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
              settings.filter_waveform ? 'bg-blue-600' : 'bg-gray-600'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                settings.filter_waveform ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {settings.filter_waveform && (
          <>
            <div className="mb-4">
              <label className="block text-xs font-semibold text-gray-400 mb-1">
                Highpass: {settings.filter_highpass} Hz
              </label>
              <input
                type="number"
                min="0.01"
                max={settings.filter_lowpass - 0.01}
                step="0.1"
                value={settings.filter_highpass}
                onChange={(e) => {
                  const v = parseFloat(e.target.value);
                  if (!isNaN(v) && v > 0) onSettingsChange({ ...settings, filter_highpass: v });
                }}
                className="w-full px-2 py-1 bg-[#202530] border border-gray-600 rounded text-gray-300 text-sm"
              />
            </div>
            <div className="mb-4">
              <label className="block text-xs font-semibold text-gray-400 mb-1">
                Lowpass: {settings.filter_lowpass} Hz
              </label>
              <input
                type="number"
                min={settings.filter_highpass + 0.01}
                max="49"
                step="0.1"
                value={settings.filter_lowpass}
                onChange={(e) => {
                  const v = parseFloat(e.target.value);
                  if (!isNaN(v) && v > settings.filter_highpass) onSettingsChange({ ...settings, filter_lowpass: v });
                }}
                className="w-full px-2 py-1 bg-[#202530] border border-gray-600 rounded text-gray-300 text-sm"
              />
            </div>
          </>
        )}
      </div>

      {/* Spectrogram Controls */}
      <div className="border-t border-gray-700 pt-4 mt-4">
        <h3 className="text-sm font-bold text-gray-300 mb-3">Spectrogram</h3>

        <div className="flex items-center justify-between mb-4">
          <label className="text-sm font-semibold text-gray-400">Show Spectrogram</label>
          <button
            onClick={() => onSettingsChange({ ...settings, show_spectrogram: !settings.show_spectrogram })}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
              settings.show_spectrogram ? 'bg-blue-600' : 'bg-gray-600'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                settings.show_spectrogram ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {settings.show_spectrogram && (
          <>
            <div className="mb-4">
              <label className="block text-xs font-semibold text-gray-400 mb-1">
                Freq Min: {settings.spectrogram_freq_min} Hz
              </label>
              <input
                type="number"
                min="0"
                max={settings.spectrogram_freq_max - 1}
                step="0.5"
                value={settings.spectrogram_freq_min}
                onChange={(e) => onSettingsChange({ ...settings, spectrogram_freq_min: parseFloat(e.target.value) })}
                className="w-full px-2 py-1 bg-[#202530] border border-gray-600 rounded text-gray-300 text-sm"
              />
            </div>

            <div className="mb-4">
              <label className="block text-xs font-semibold text-gray-400 mb-1">
                Freq Max: {settings.spectrogram_freq_max} Hz
              </label>
              <input
                type="number"
                min={settings.spectrogram_freq_min + 1}
                max="100"
                step="0.5"
                value={settings.spectrogram_freq_max}
                onChange={(e) => onSettingsChange({ ...settings, spectrogram_freq_max: parseFloat(e.target.value) })}
                className="w-full px-2 py-1 bg-[#202530] border border-gray-600 rounded text-gray-300 text-sm"
              />
            </div>

            <div className="flex items-center justify-between">
              <label className="text-sm font-semibold text-gray-400">Log Y-Axis</label>
              <button
                onClick={() => onSettingsChange({ ...settings, spectrogram_log_y: !settings.spectrogram_log_y })}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
                  settings.spectrogram_log_y ? 'bg-blue-600' : 'bg-gray-600'
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    settings.spectrogram_log_y ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default ControlPanel;
