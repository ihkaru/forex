export interface TickData {
  symbol: string;
  bid: number;
  ask: number;
  spreadPips: number;
  timestamp: string;
}

export interface CandleData {
  symbol: string;
  timeframe: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  timestamp: string;
}

export interface MarketStreamPort {
  subscribeTicks(symbol: string, onTick: (tick: TickData) => void): () => void;
  subscribeCandles(symbol: string, timeframe: string, onCandle: (candle: CandleData) => void): () => void;
}
