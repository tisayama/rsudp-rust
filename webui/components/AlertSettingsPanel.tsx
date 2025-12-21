'use client';

import React, { useEffect, useState } from 'react';
import { AlertSettings } from '../lib/types';

const AlertSettingsPanel: React.FC = () => {
  const [settings, setSettings] = useState<AlertSettings | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    fetch('http://localhost:8080/api/alerts/settings')
      .then(res => res.json())
      .then(data => setSettings(data))
      .catch(err => console.error('Failed to fetch alert settings:', err));
  }, []);

  const saveSettings = () => {
    if (!settings) return;
    setIsSaving(true);
    fetch('http://localhost:8080/api/alerts/settings', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings),
    })
      .then(() => setIsSaving(false))
      .catch(err => {
        console.error('Failed to save alert settings:', err);
        setIsSaving(false);
      });
  };

  if (!settings) return null;

  return (
    <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
      <h2 className="text-xl font-bold mb-4 text-gray-800">Alert Notification</h2>
      
      <div className="space-y-4 mb-6">
        <div className="flex items-center justify-between">
          <label className="text-sm font-semibold text-gray-700">Audio Alerts</label>
          <button
            onClick={() => setSettings({ ...settings, audio_enabled: !settings.audio_enabled })}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.audio_enabled ? 'bg-blue-600' : 'bg-gray-200'}`}
          >
            <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.audio_enabled ? 'translate-x-6' : 'translate-x-1'}`} />
          </button>
        </div>

        <div className="flex items-center justify-between">
          <label className="text-sm font-semibold text-gray-700">Email Alerts</label>
          <button
            onClick={() => setSettings({ ...settings, email_enabled: !settings.email_enabled })}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${settings.email_enabled ? 'bg-blue-600' : 'bg-gray-200'}`}
          >
            <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${settings.email_enabled ? 'translate-x-6' : 'translate-x-1'}`} />
          </button>
        </div>
      </div>

      {settings.email_enabled && (
        <div className="space-y-3 mb-6 p-4 bg-slate-50 rounded-lg border border-slate-100">
          <div>
            <label className="block text-[10px] font-black uppercase text-slate-400 mb-1">SMTP Host</label>
            <input 
              type="text" 
              value={settings.smtp_host} 
              onChange={e => setSettings({...settings, smtp_host: e.target.value})}
              className="w-full text-xs p-2 rounded border border-slate-200"
            />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-[10px] font-black uppercase text-slate-400 mb-1">SMTP Port</label>
              <input 
                type="number" 
                value={settings.smtp_port} 
                onChange={e => setSettings({...settings, smtp_port: parseInt(e.target.value)})}
                className="w-full text-xs p-2 rounded border border-slate-200"
              />
            </div>
            <div>
              <label className="block text-[10px] font-black uppercase text-slate-400 mb-1">User</label>
              <input 
                type="text" 
                value={settings.smtp_user} 
                onChange={e => setSettings({...settings, smtp_user: e.target.value})}
                className="w-full text-xs p-2 rounded border border-slate-200"
              />
            </div>
          </div>
          <div>
            <label className="block text-[10px] font-black uppercase text-slate-400 mb-1">Recipient</label>
            <input 
              type="email" 
              value={settings.email_recipient} 
              onChange={e => setSettings({...settings, email_recipient: e.target.value})}
              className="w-full text-xs p-2 rounded border border-slate-200"
            />
          </div>
        </div>
      )}

      <button
        onClick={saveSettings}
        disabled={isSaving}
        className="w-full py-2 bg-slate-900 text-white rounded-lg font-bold text-sm hover:bg-slate-800 disabled:opacity-50 transition-colors"
      >
        {isSaving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  );
};

export default AlertSettingsPanel;
