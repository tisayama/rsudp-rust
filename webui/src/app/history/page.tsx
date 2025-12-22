'use client';

import { useEffect, useState } from 'react';
import { AlertEvent } from '../../../lib/types';
import Link from 'next/link';

export default function HistoryPage() {
  const [history, setHistory] = useState<AlertEvent[]>([]);

  useEffect(() => {
    fetch('http://localhost:8080/api/alerts')
      .then(res => res.json())
      .then(data => setHistory(data))
      .catch(err => console.error('Failed to fetch history:', err));
  }, []);

  return (
    <main className="min-h-screen bg-slate-50 p-4 md:p-8">
      <div className="max-w-5xl mx-auto space-y-8">
        <header className="flex justify-between items-center">
          <h1 className="text-3xl font-black text-slate-900 tracking-tight italic">
            ALERT <span className="text-blue-600">HISTORY</span>
          </h1>
          <Link href="/" className="px-4 py-2 bg-white rounded-full shadow-sm border border-slate-200 text-sm font-bold text-slate-600 hover:bg-slate-100 transition-colors">
            Back to Monitor
          </Link>
        </header>

        <div className="grid gap-6">
          {history.length === 0 ? (
            <div className="h-64 flex flex-col items-center justify-center bg-white border border-slate-200 rounded-2xl text-slate-400">
              <p className="font-medium text-lg">No alert history in the last 24 hours</p>
            </div>
          ) : (
            history.sort((a, b) => new Date(b.trigger_time).getTime() - new Date(a.trigger_time).getTime()).map(event => (
              <div key={event.id} className="bg-white p-6 rounded-2xl shadow-sm border border-slate-200 flex flex-col md:flex-row gap-6">
                <div className="flex-1 space-y-4">
                  <div className="flex justify-between items-start">
                    <div>
                      <h2 className="text-xl font-bold text-slate-900">{event.channel} Alert</h2>
                      <p className="text-sm text-slate-500">{new Date(event.trigger_time).toLocaleString()}</p>
                    </div>
                    <div className="flex flex-col items-end gap-2">
                      <span className="px-3 py-1 bg-rose-100 text-rose-600 rounded-full text-xs font-bold uppercase">
                        Max Ratio: {event.max_ratio.toFixed(2)}
                      </span>
                    </div>
                  </div>
                  {event.message && (
                    <div className="p-3 bg-blue-50 border border-blue-100 rounded-xl">
                      <p className="text-blue-700 font-bold">{event.message}</p>
                    </div>
                  )}
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div className="bg-slate-50 p-3 rounded-xl">
                      <p className="text-slate-400 font-bold uppercase text-[10px]">Triggered</p>
                      <p className="text-slate-700 font-mono">{new Date(event.trigger_time).toLocaleTimeString()}</p>
                    </div>
                    <div className="bg-slate-50 p-3 rounded-xl">
                      <p className="text-slate-400 font-bold uppercase text-[10px]">Reset</p>
                      <p className="text-slate-700 font-mono">{event.reset_time ? new Date(event.reset_time).toLocaleTimeString() : 'Active'}</p>
                    </div>
                  </div>
                </div>
                {event.snapshot_path && (
                  <div className="md:w-64 aspect-video bg-slate-100 rounded-xl overflow-hidden border border-slate-200">
                    {/* eslint-disable-next-line @next/next/no-img-element */}
                    <img 
                      src={`http://localhost:8080/images/alerts/${event.snapshot_path}`} 
                      alt="Waveform snapshot"
                      className="w-full h-full object-cover cursor-zoom-in hover:scale-105 transition-transform"
                      onClick={() => window.open(`http://localhost:8080/images/alerts/${event.snapshot_path}`, '_blank')}
                    />
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </main>
  );
}
