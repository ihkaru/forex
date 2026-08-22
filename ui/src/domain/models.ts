// ============================================================================
// DOMAIN MODELS (The Hexagon Core - Pure TypeScript, Zero I/O)
// ============================================================================

export type SignalAction = 'BUY_LIMIT' | 'SELL_LIMIT' | 'BUY_STOP' | 'SELL_STOP';
export type SignalStatus = 'PENDING' | 'ACTIVE' | 'TARGET_HIT' | 'SL_HIT' | 'EXPIRED';

export type MarketDataSource = 'DukascopyEcn' | 'Mt5BrokerLive' | 'CtraderOpenApi' | 'SyntheticTest';

export interface Candle {
  time: number; // Unix timestamp in seconds
  open: number;
  high: number;
  low: number;
  close: number;
  volume?: number;
  source?: MarketDataSource;
}

export interface SwingPoint {
  index: number;
  price: number;
  isHigh: boolean;
  time: number;
}

export interface PolaNFormation {
  l1: number;
  h1: number;
  l2: number;
  h2?: number;
  isBullish: boolean;
  goldenZoneEntry: number;
  structuralStopLoss: number;
  takeProfit1: number;
  takeProfit2: number;
  riskRewardRatio: string;
  impulsePips: number;
}

export interface Signal {
  id: string;
  symbol: string;
  action: SignalAction;
  entryPrice: number;
  stopLoss: number;
  takeProfit1: number;
  takeProfit2: number;
  riskRewardRatio: number;
  valuedPipsEstimate?: number;
  confidenceScore?: number;
  strategyName: string;
  rationale: string;
  status: SignalStatus | string;
  createdAt: string;
}

export interface BacktestReport {
  symbol: string;
  totalTrades: number;
  winningTrades: number;
  losingTrades: number;
  winRatePercent: number;
  totalRawPips: number;
  totalValuedPips: number;
  profitFactor: number;
  recoveryFactor: number;
  isTfQualified: boolean;
}

export interface EdaReport {
  symbol: string;
  totalCandles: number;
  durationDays: number;
  mathematicalIntegrityPct: number;
  avgBarRangePips: number;
  maxSingleBarPips: number;
  healthStatus: string;
}
