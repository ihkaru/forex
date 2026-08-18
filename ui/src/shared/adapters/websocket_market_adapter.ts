import type { CandleData, MarketStreamPort, TickData } from '../ports/market_stream_port';

export class WebSocketMarketAdapter implements MarketStreamPort {
  private wsUrl: string;

  constructor(wsUrl: string = 'ws://127.0.0.1:8080/ws/live') {
    this.wsUrl = wsUrl;
  }

  subscribeTicks(symbol: string, onTick: (tick: TickData) => void): () => void {
    // Simulasi live stream / WebSocket connection
    const interval = setInterval(() => {
      const basePrice = symbol.includes('XAU') ? 2385.0 : 1.0850;
      const jitter = (Math.random() - 0.5) * 0.0004;
      const bid = +(basePrice + jitter).toFixed(5);
      const ask = +(bid + 0.00015).toFixed(5);

      onTick({
        symbol,
        bid,
        ask,
        spreadPips: 1.5,
        timestamp: new Date().toISOString(),
      });
    }, 1000);

    return () => clearInterval(interval);
  }

  subscribeCandles(symbol: string, timeframe: string, onCandle: (candle: CandleData) => void): () => void {
    const interval = setInterval(() => {
      onCandle({
        symbol,
        timeframe,
        open: 1.0845,
        high: 1.0862,
        low: 1.084,
        close: 1.0855,
        volume: 1400,
        timestamp: new Date().toISOString(),
      });
    }, 5000);

    return () => clearInterval(interval);
  }
}
