// ============================================================================
// DOMAIN MODELS (The Hexagon Core - Pure TypeScript, Zero I/O)
// ============================================================================

export type SignalAction = 'BUY_LIMIT' | 'SELL_LIMIT' | 'BUY_STOP' | 'SELL_STOP';
export type SignalStatus = 'PENDING' | 'ACTIVE' | 'TARGET_HIT' | 'SL_HIT' | 'EXPIRED';

export type ChartType =
  | 'CANDLES'
  | 'VOLUME_CANDLES'
  | 'HEIKIN_ASHI'
  | 'LINE'
  | 'AREA'
  | 'BARS'
  | 'BASELINE';

export interface ChartTypeOption {
  id: ChartType;
  name: string;
  shortLabel: string;
  category: 'PRICE' | 'LINE' | 'SYNTHETIC';
  categoryLabel: string;
  description: string;
  hotkey?: string;
  isDerived?: boolean;
}

export type MarketDataSource = 'DukascopyEcn' | 'Mt5BrokerLive' | 'MrgMetaTrader4' | 'CtraderOpenApi' | 'SyntheticTest';


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
  timeframe?: string;
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
  total_candles?: number;
  durationDays: number;
  mathematicalIntegrityPct: number;
  data_health_score?: number;
  avgBarRangePips: number;
  avg_spread_pips?: number;
  maxSingleBarPips: number;
  healthStatus: string;
}

export interface DeltaSyncReport {
  symbol: string;
  timeframe: string;
  source: MarketDataSource;
  previousWatermark?: number;
  newWatermark?: number;
  syncedBarsCount: number;
  durationMs: number;
  isUpToDate: boolean;
  message?: string;
}

export type ExecutionHudStatus = 'SCANNING' | 'PENDING' | 'RUNNING' | 'SETTLED';

export interface MarketScanContext {
  trend: 'BULLISH' | 'BEARISH' | 'NEUTRAL';
  fastEma: number;
  slowEma: number;
  rsi: number;
  sessionName: string;
  isSessionActive: boolean;
  waitingReason: string;
}

export interface PendingOrderState {
  tradeId: string;
  action: string;
  entryPrice: number;
  currentPrice: number;
  stopLoss: number;
  takeProfit: number;
  distancePips: number;
  postedTime?: number;
  slaMinutes?: number;
}

export interface RunningPositionState {
  tradeId: string;
  action: string;
  openPrice: number;
  currentPrice: number;
  stopLoss: number;
  takeProfit: number;
  floatingPips: number;
  floatingValuedPips: number;
  progressToTpPct: number;
  heldBarsCount: number;
  isProfit: boolean;
}

export interface SettledTradeState {
  tradeId: string;
  action: string;
  openPrice: number;
  closePrice: number;
  pnlPips: number;
  valuedPips: number;
  isWin: boolean;
  exitReason: string;
}

export interface ExecutionHudData {
  status: ExecutionHudStatus;
  symbol: string;
  strategyName: string;
  scanContext?: MarketScanContext;
  pendingOrder?: PendingOrderState | null;
  pendingSignal?: Signal | null;
  runningPosition?: RunningPositionState | null;
  settledTrade?: SettledTradeState | null;
}


/**
 * Normalizes any symbol format ('EUR/USD', 'eurusd', 'EUR_USD') to standard 'EURUSD'.
 */
export function normalizeSymbol(symbol?: string): string {
  return symbol ? symbol.replace(/[^A-Za-z0-9]/g, '').toUpperCase() : '';
}

/**
 * Returns the exact pip scale multiplier for any asset class (Forex, JPY, Gold, Crypto, Indices).
 */
export function getPipMultiplier(symbol?: string): number {
  const norm = normalizeSymbol(symbol);
  if (norm.includes('JPY')) return 100;
  if (norm === 'XAUUSD' || norm.includes('GOLD')) return 10;
  if (norm.includes('BTC') || norm.includes('ETH')) return 1;
  if (norm.includes('US30') || norm.includes('NAS100') || norm.includes('SPX')) return 1;
  return 10000;
}

/**
 * Returns Traders Family Valued Pips (VP) tier conversion multiplier.
 */
export function getValuedPipsMultiplier(symbol?: string): number {
  const norm = normalizeSymbol(symbol);
  if (norm === 'XAUUSD' || norm.includes('GOLD')) return 0.5; // Tier 4 Gold (1 pip = 0.5 VP)
  if (['EURUSD', 'GBPUSD', 'AUDUSD', 'NZDUSD'].includes(norm)) return 2.0; // Tier 1 Major (1 pip = 2.0 VP)
  return 1.0; // Tier 2 & 3 Standard (1 pip = 1.0 VP)
}



