/**
 * Returns the backend API origin (e.g. "http://192.168.1.100:8080").
 *
 * Resolution order:
 *  1. NEXT_PUBLIC_API_HOST env var (set at build or in .env.local)
 *  2. Same hostname as the current page, port 8080
 *  3. Fallback to http://localhost:8080 (SSR / non-browser)
 */
export function getBackendOrigin(): string {
  if (typeof window === 'undefined') return 'http://localhost:8080';
  const envHost = process.env.NEXT_PUBLIC_API_HOST;
  if (envHost) return envHost.replace(/\/$/, '');
  return `${window.location.protocol}//${window.location.hostname}:8080`;
}

/**
 * Returns the WebSocket URL for live data streaming.
 */
export function getWsUrl(): string {
  if (typeof window === 'undefined') return 'ws://localhost:8080/ws';
  const envHost = process.env.NEXT_PUBLIC_API_HOST;
  if (envHost) {
    const base = envHost.replace(/\/$/, '');
    const wsProto = base.startsWith('https') ? 'wss' : 'ws';
    return `${wsProto}://${new URL(base).host}/ws`;
  }
  const wsProto = window.location.protocol === 'https:' ? 'wss' : 'ws';
  return `${wsProto}://${window.location.hostname}:8080/ws`;
}
