import type { Candle, MarketDataSource } from '../../domain/models';
import type { IMarketDataPort } from '../../ports';

export class RestMarketDataAdapter implements IMarketDataPort {
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
    const candles = await this.getCandles(symbol, 'H1', 1, source);
    if (candles.length > 0) {
      return candles[candles.length - 1].close;
    }
    return 1.0850;
  }
}

