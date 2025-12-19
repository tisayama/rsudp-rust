import { useEffect, useRef, useState, useCallback } from 'react';
import { WsMessage } from '../lib/types';

export const useWebSocket = (url: string) => {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null);
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();
  const reconnectDelayRef = useRef(1000);

  const connect = useCallback(() => {
    if (socketRef.current?.readyState === WebSocket.CONNECTING || socketRef.current?.readyState === WebSocket.OPEN) return;

    const socket = new WebSocket(url);
    socket.binaryType = 'arraybuffer';
    socketRef.current = socket;

    socket.onopen = () => {
      setIsConnected(true);
      reconnectDelayRef.current = 1000;
      console.log('Connected to WebSocket');
    };

    socket.onmessage = async (event) => {
      if (event.data instanceof ArrayBuffer) {
        const view = new DataView(event.data);
        const type = view.getUint8(0);

        if (type === 0) {
          // Waveform
          let offset = 1;
          const channelIdLen = view.getUint8(offset);
          offset += 1;
          const channelId = new TextDecoder().decode(event.data.slice(offset, offset + channelIdLen));
          offset += channelIdLen;
          const tsMicros = BigInt(new BigInt64Array(event.data.slice(offset, offset + 8))[0]);
          offset += 8;
          const sampleRate = view.getFloat32(offset, true);
          offset += 4;
          const samplesCount = view.getUint32(offset, true);
          offset += 4;
          const samples = new Float32Array(event.data.slice(offset));

          setLastMessage({
            type: 'Waveform',
            data: {
              channel_id: channelId,
              timestamp: new Date(Number(tsMicros / 1000n)).toISOString(),
              samples: Array.from(samples),
              sample_rate: sampleRate,
            }
          });
        } else if (type === 1) {
          // Alert
          const json = new TextDecoder().decode(event.data.slice(1));
          const alert = JSON.parse(json);
          setLastMessage({ type: 'Alert', data: alert });
        }
      }
    };

    socket.onclose = () => {
      setIsConnected(false);
      const delay = reconnectDelayRef.current;
      console.log(`Disconnected. Reconnecting in ${delay}ms...`);
      reconnectTimeoutRef.current = setTimeout(() => {
        reconnectDelayRef.current = Math.min(delay * 2, 30000); // Max 30s
        connect();
      }, delay);
    };

    socket.onerror = (error) => {
      console.error('WebSocket Error:', error);
      socket.close();
    };
  }, [url]);

  useEffect(() => {
    connect();
    return () => {
      if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current);
      if (socketRef.current) socketRef.current.close();
    };
  }, [connect]);

  return { isConnected, lastMessage };
};
