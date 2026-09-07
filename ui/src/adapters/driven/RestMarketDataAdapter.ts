import type { Candle, MarketDataSource } from '../../domain/models';
import type { IMarketDataPort } from '../../ports';

export class RestMarketDataAdapter implements IMarketDataPort {
  private socket: WebSocket | null = null;
  private isClosed = false;
  private readonly tickCache = new Map<string, Candle>();

  constructor(private readonly baseUrl: string = 'http://127.0.0.1:5000/api') {}

  async getCandles(
    symbol: string,
    timeframe: string = 'H1',
    limit: number = 300,
    source: MarketDataSource | string = 'dukascopy'
  ): Promise<Candle[]> {
    try {
      const srcParam = typeof source === 'string' ? source.toLowerCase() : 'dukascopy';
      const res = await fetch(
        `${this.baseUrl}/market/candles/${symbol}?source=${srcParam}&timeframe=${timeframe}&limit=${limit}`
      );
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.warn(`[RestMarketDataAdapter] Gagal fetch candle dari API, menggunakan fallback memory: ${e}`);
    }

    // Fallback ke window.REAL_MARKET_DATA jika API belum tersedia
    const globalData = (window as any).REAL_MARKET_DATA;
    if (globalData && globalData[symbol]) {
      return globalData[symbol];
    }
    return [];
  }

  async getLatestPrice(symbol: string, source: MarketDataSource | string = 'dukascopy'): Promise<number> {
    const cached = this.tickCache.get(symbol);
    if (cached) {
      return cached.close;
    }
    const candles = await this.getCandles(symbol, 'H1', 1, source);
    if (candles.length > 0) {
      return candles[candles.length - 1].close;
    }
    return 1.0850;
  }

  async *streamCandles(symbol: string, timeframe = 'H1', source?: MarketDataSource | string): AsyncIterable<Candle> {
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
        const wsUrl = this.baseUrl.replace(/^http/, 'ws') + `/market/stream/${symbol}`;
        console.log(`[RestMarketDataAdapter] Connecting to live WebSocket: ${wsUrl}`);
        ws = new WebSocket(wsUrl);
        this.socket = ws;

        ws.onopen = () => {
          console.log(`[RestMarketDataAdapter] Connected to live WebSocket stream for ${symbol}`);
        };

        ws.onmessage = (event: MessageEvent) => {
          try {
            const payload = typeof event.data === 'string' ? JSON.parse(event.data) : event.data;
            if (!payload || typeof payload.close !== 'number') return;
            const candle: Candle = {
              time: Number(payload.time) || Math.floor(Date.now() / 1000),
              open: Number(payload.open) || payload.close,
              high: Number(payload.high) || payload.close,
              low: Number(payload.low) || payload.close,
              close: Number(payload.close),
              volume: Number(payload.volume) || 1.0,
              source: (payload.source as any) || 'MrgDemoMt4',
            };
            this.tickCache.set(symbol, candle);
            queue.push(candle);
            notify();
          } catch (err) {
            console.warn('[RestMarketDataAdapter] Error parsing WS message:', err);
          }
        };

        ws.onerror = (err) => {
          console.warn('[RestMarketDataAdapter] WebSocket error:', err);
        };

        ws.onclose = () => {
          console.log(`[RestMarketDataAdapter] WebSocket disconnected for ${symbol}`);
        };
      } catch (err) {
        console.warn('[RestMarketDataAdapter] WebSocket connection error:', err);
      }
    }

    try {
      while (!this.isClosed) {
        if (queue.length === 0) {
          await new Promise<void>((resolve) => {
            wake = resolve;
            setTimeout(resolve, 10000);
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

