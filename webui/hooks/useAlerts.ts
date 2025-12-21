import { useState, useEffect, useRef } from 'react';
import { WsMessage } from '../lib/types';

export function useAlerts(lastMessage: WsMessage | null) {
  const [activeAlerts, setActiveAlerts] = useState<Map<string, string>>(new Map());
  const [isAlerting, setIsAlerting] = useState(false);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    if (typeof window !== 'undefined') {
      audioRef.current = new Audio('/sounds/alert.wav');
      audioRef.current.loop = true;
    }
  }, []);

  useEffect(() => {
    if (!lastMessage) return;

    if (lastMessage.type === 'AlertStart') {
      const { id, channel } = lastMessage.data;
      setActiveAlerts((prev) => {
        const next = new Map(prev);
        next.set(id, channel);
        return next;
      });
    } else if (lastMessage.type === 'AlertEnd') {
      const { id } = lastMessage.data;
      setActiveAlerts((prev) => {
        const next = new Map(prev);
        next.delete(id);
        return next;
      });
    }
  }, [lastMessage]);

  useEffect(() => {
    const active = activeAlerts.size > 0;
    setIsAlerting(active);

    if (active) {
      audioRef.current?.play().catch((e) => console.warn('Audio play failed:', e));
    } else {
      audioRef.current?.pause();
      if (audioRef.current) audioRef.current.currentTime = 0;
    }
  }, [activeAlerts]);

  return { isAlerting, activeAlerts };
}
