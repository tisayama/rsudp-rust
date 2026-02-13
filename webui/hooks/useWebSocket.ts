import { useEffect, useRef, useState, useCallback } from 'react';
import { WsMessage, SpectrogramPacket } from '../lib/types';

export interface UseWebSocketOptions {
  onSpectrogramData?: (batchTimestamp: number, channelId: string, columns: Uint8Array[], frequencyBins: number, sampleRate: number, hopDuration: number) => void;
}

export const useWebSocket = (url: string, options?: UseWebSocketOptions) => {
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WsMessage | null>(null);
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const reconnectDelayRef = useRef(1000);
  const lastTimestampRef = useRef<string | null>(null);
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const connect = useCallback(() => {
    if (socketRef.current?.readyState === WebSocket.CONNECTING || socketRef.current?.readyState === WebSocket.OPEN) return;

    console.log(`[WS] Connecting to ${url} at ${new Date().toISOString()}`);
    const socket = new WebSocket(url);
    socket.binaryType = 'arraybuffer';
    socketRef.current = socket;

    socket.onopen = () => {
      setIsConnected(true);
      reconnectDelayRef.current = 1000;
      console.log(`[WS] Connected at ${new Date().toISOString()}`);

      // Send BackfillRequest
      const request: Record<string, unknown> = { type: 'BackfillRequest' };
      if (lastTimestampRef.current) {
        request.last_timestamp = lastTimestampRef.current;
      }
      socket.send(JSON.stringify(request));
    };

    socket.onmessage = async (event) => {
      if (typeof event.data === 'string') {
        try {
          const msg = JSON.parse(event.data) as WsMessage;
          setLastMessage(msg);
        } catch (e) {
          console.error('[WS] Failed to parse WebSocket message:', e);
        }
        return;
      }

      if (event.data instanceof ArrayBuffer) {
        const view = new DataView(event.data);
        const type = view.getUint8(0);

        if (type === 0x00) {
          // Waveform binary packet
          let offset = 1;
          const channelIdLen = view.getUint8(offset);
          offset += 1;
          const channelId = new TextDecoder().decode(event.data.slice(offset, offset + channelIdLen));
          offset += channelIdLen;
          const tsMicros = new BigInt64Array(event.data.slice(offset, offset + 8))[0];
          offset += 8;
          const sampleRate = view.getFloat32(offset, true);
          offset += 4;
          const samplesCount = view.getUint32(offset, true);
          offset += 4;
          const samples = new Float32Array(event.data.slice(offset, offset + samplesCount * 4));

          // Update last_timestamp for reconnection backfill
          const tsMs = Number(tsMicros / 1000n) + (samplesCount / sampleRate) * 1000;
          lastTimestampRef.current = new Date(tsMs).toISOString();

          setLastMessage({
            type: 'Waveform',
            data: {
              channel_id: channelId,
              timestamp: new Date(Number(tsMicros / 1000n)).toISOString(),
              samples: Array.from(samples),
              sample_rate: sampleRate,
            }
          });
        } else if (type === 0x03) {
          // Spectrogram binary packet
          let offset = 1;
          const channelIdLen = view.getUint8(offset);
          offset += 1;
          const channelId = new TextDecoder().decode(event.data.slice(offset, offset + channelIdLen));
          offset += channelIdLen;
          // timestamp i64le (microseconds since epoch)
          const specTsMicros = new BigInt64Array(event.data.slice(offset, offset + 8))[0];
          const batchTimestamp = Number(specTsMicros / 1000n); // convert to milliseconds
          offset += 8;
          const sampleRate = view.getFloat32(offset, true);
          offset += 4;
          const hopDuration = view.getFloat32(offset, true);
          offset += 4;
          const frequencyBins = view.getUint16(offset, true);
          offset += 2;
          const columnsCount = view.getUint16(offset, true);
          offset += 2;

          // Extract columns (column-major: each column has frequencyBins u8 values)
          const columns: Uint8Array[] = [];
          for (let col = 0; col < columnsCount; col++) {
            const colStart = offset + col * frequencyBins;
            const colData = new Uint8Array(event.data.slice(colStart, colStart + frequencyBins));
            columns.push(colData);
          }

          // Notify via callback (batchTimestamp = start time of first column in this batch)
          optionsRef.current?.onSpectrogramData?.(batchTimestamp, channelId, columns, frequencyBins, sampleRate, hopDuration);
        } else if (type === 1) {
          // Alert
          const json = new TextDecoder().decode(event.data.slice(1));
          const alert = JSON.parse(json);
          setLastMessage({ type: 'Alert', data: alert });
        } else if (type === 2) {
          // Intensity
          const json = new TextDecoder().decode(event.data.slice(1));
          const intensity = JSON.parse(json);
          setLastMessage({ type: 'Intensity', data: intensity });
        }
      }
    };

    socket.onclose = (event) => {
      setIsConnected(false);
      const delay = reconnectDelayRef.current;
      console.log(`[WS] Disconnected at ${new Date().toISOString()} code=${event.code} reason="${event.reason}" wasClean=${event.wasClean}. Reconnecting in ${delay}ms...`);
      reconnectTimeoutRef.current = setTimeout(() => {
        reconnectDelayRef.current = Math.min(delay * 2, 30000);
        connect();
      }, delay);
    };

    socket.onerror = (error) => {
      console.error(`[WS] Error at ${new Date().toISOString()}:`, error);
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
