import type { Candle } from '../../domain/models';
import type { IMarketDataPort } from '../../ports';

export class RestMarketDataAdapter implements IMarketDataPort {
  constructor(private readonly baseUrl: string = 'http://127.0.0.1:5000/api') {}

  async getCandles(symbol: string, timeframe: string = 'H1', limit: number = 300): Promise<Candle[]> {
    try {
      const res = await fetch(`${this.baseUrl}/market/candles/${symbol}?timeframe=${timeframe}&limit=${limit}`);
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

  async getLatestPrice(symbol: string): Promise<number> {
    const candles = await this.getCandles(symbol, 'H1', 1);
    if (candles.length > 0) {
      return candles[candles.length - 1].close;
    }
    return 1.0850;
  }
}
