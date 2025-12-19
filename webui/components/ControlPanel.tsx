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
    <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
      <h2 className="text-xl font-bold mb-4 text-gray-800">Settings</h2>
      
      <div className="mb-6">
        <label className="block text-sm font-semibold text-gray-700 mb-2">Channels</label>
        <div className="flex flex-wrap gap-2">
          {availableChannels.map(channel => (
            <button
              key={channel}
              onClick={() => toggleChannel(channel)}
              className={`px-3 py-1.5 rounded-full text-xs font-bold transition-colors ${
                settings.active_channels.includes(channel)
                  ? 'bg-blue-600 text-white shadow-md'
                  : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
              }`}
            >
              {channel}
            </button>
          ))}
        </div>
      </div>

      <div className="mb-6">
        <label className="block text-sm font-semibold text-gray-700 mb-2">
          Time Window: {settings.window_seconds}s
        </label>
        <input
          type="range"
          min="10"
          max="300"
          step="10"
          value={settings.window_seconds}
          onChange={(e) => onSettingsChange({ ...settings, window_seconds: parseInt(e.target.value) })}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
        />
      </div>

      <div className="flex items-center justify-between">
        <label className="text-sm font-semibold text-gray-700">Auto-scale</label>
        <button
          onClick={() => onSettingsChange({ ...settings, auto_scale: !settings.auto_scale })}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
            settings.auto_scale ? 'bg-blue-600' : 'bg-gray-200'
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              settings.auto_scale ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>
    </div>
  );
};

export default ControlPanel;
