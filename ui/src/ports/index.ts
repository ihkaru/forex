import type { Candle, Signal, BacktestReport, EdaReport } from '../domain/models';
import type { SimulatedTrade } from './layers';

export * from './layers';

export interface StrategyDescriptor {
  id: string;
  name: string;
  code: string;
  description: string;
  category: string;
  author: string;
  winRatePct: number;
  profitFactor: number;
  recoveryFactor: number;
  sharpeRatio: number;
  sortinoRatio: number;
  calmarRatio: number;
  wferPct: number;
  isTfCompliant: boolean;
  supportedSymbols: string[];
  isSpecialist?: boolean;
  specialistLabel?: string | null;
}

export interface MonteCarloPercentilePoint {
  trade_index: number;
  p5_worst: number;
  p25: number;
  p50_median: number;
  p75: number;
  p95_best: number;
  actual_equity: number;
}

export interface MonteCarloReport {
  symbol: string;
  strategy_id: string;
  iterations: number;
  original_trades_count: number;
  risk_of_ruin_pct: number;
  median_max_dd_pct: number;
  worst_case_max_dd_pct: number;
  median_ending_vp: number;
  worst_case_ending_vp: number;
  confidence_interval_95: [number, number];
  equity_paths: MonteCarloPercentilePoint[];
}

export interface IMarketDataPort {
  getCandles(symbol: string, timeframe?: string, limit?: number): Promise<Candle[]>;
  getLatestPrice(symbol: string): Promise<number>;
}

export interface ISignalPublisherPort {
  broadcastToTraderFamily(signal: Signal): Promise<{ success: boolean; postId: string }>;
}

export interface IBacktestPort {
  runBacktest(strategyId?: string): Promise<{
    reports: BacktestReport[];
    totalValuedPips: number;
    portfolioWinRatePct: number;
    wferPct: number;
    isTfQualified: boolean;
  }>;
  getTrades(symbol: string, strategyId?: string): Promise<SimulatedTrade[]>;
}

export interface IStrategyPort {
  getStrategies(): Promise<StrategyDescriptor[]>;
}

export interface IMonteCarloPort {
  getMonteCarloReport(symbol: string): Promise<MonteCarloReport>;
}

export interface IEdaHealthPort {
  getEdaHealth(symbol: string): Promise<EdaReport>;
}

export interface IDeltaSyncPort {
  syncPairDelta(symbol: string, timeframe?: string, source?: MarketDataSource): Promise<DeltaSyncReport>;
}
