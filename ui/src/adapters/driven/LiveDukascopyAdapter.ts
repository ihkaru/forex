import type { Candle } from '../../domain/models';
import type { IMarketDataPort } from '../../ports';
import { RestMarketDataAdapter } from './RestMarketDataAdapter';

/** Historical bootstrap plus optional live Dukascopy tick stream. */
export class LiveDukascopyAdapter implements IMarketDataPort {
  private readonly fallback: RestMarketDataAdapter;
  private readonly tickCache = new Map<string, Candle>();
  private socket: WebSocket | null = null;
  private isClosed = false;

  constructor(private readonly apiBaseUrl: string = 'http://127.0.0.1:5000/api') {
    this.fallback = new RestMarketDataAdapter(apiBaseUrl);
  }

  async getCandles(symbol: string, timeframe = 'H1', limit = 300): Promise<Candle[]> {
    return this.fallback.getCandles(symbol, timeframe, limit);
  }

  async getLatestPrice(symbol: string): Promise<number> {
    return this.tickCache.get(symbol)?.close ?? this.fallback.getLatestPrice(symbol);
  }

  async *streamCandles(symbol: string, timeframe = 'M1'): AsyncIterable<Candle> {
    this.isClosed = false;
    const queue: Candle[] = [];
    let wake: (() => void) | undefined;

    const notify = (): void => {
      const resolve = wake;
      wake = undefined;
      resolve?.();
    };

    let ws: WebSocket | null = null;

    if (typeof WebSocket !== 'undefined') {
      try {
        const wsUrl = this.apiBaseUrl.replace(/^http/, 'ws') + `/market/stream/${symbol}`;
        ws = new WebSocket(wsUrl);
        this.socket = ws;

        ws.onmessage = (event: MessageEvent) => {
          try {
            const payload = typeof event.data === 'string' ? JSON.parse(event.data) : event.data;
            if (!payload || typeof payload.close !== 'number') return;
            const candle: Candle = {
              time: Number(payload.time) || Math.floor(Date.now() / 1000),
              open: Number(payload.open) || payload.close,
              high: Number(payload.high) || payload.close,
              low: Number(payload.low) || payload.close,
              close: payload.close,
              volume: Number(payload.volume) || 1.0,
              source: 'DukascopyEcn',
            };
            this.tickCache.set(symbol, candle);
            queue.push(candle);
            notify();
          } catch {}
        };

        ws.onerror = () => {
          // Graceful silent recovery when backend live stream is idle/offline
        };

        ws.onclose = () => {
          // Silent close handling
        };
      } catch {
        // Suppress initial connection exception
      }
    }

    try {
      while (!this.isClosed) {
        if (queue.length === 0) {
          await new Promise<void>((resolve) => {
            wake = resolve;
            setTimeout(resolve, 10000); // 10s idle sleep
          });
        }
        while (queue.length > 0) {
          const candle = queue.shift();
          if (candle) yield candle;
        }
      }
    } finally {
      if (ws) {
        try {
          ws.close();
        } catch {}
      }
      if (this.socket === ws) {
        this.socket = null;
      }
    }
  }

  close(): void {
    this.isClosed = true;
    if (this.socket) {
      try {
        this.socket.close();
      } catch {}
      this.socket = null;
    }
  }
}
