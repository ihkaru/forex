import type { Candle, Signal, BacktestReport, EdaReport, MarketDataSource, DeltaSyncReport } from '../domain/models';
import type { SimulatedTrade } from './layers';

export * from './layers';
export * from './IReplayKpiPort';


export interface StrategyParameterSchema {
  key: string;
  label: string;
  param_type: string;
  default_value: any;
  min?: number | null;
  max?: number | null;
  step?: number | null;
  options?: string[] | null;
  group: string;
  tooltip?: string | null;
}

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
  activeParametersSummary?: string;
  parameters?: StrategyParameterSchema[];
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

export interface TfScorecardPillar {
  code?: string;
  name: string;
  score: number;
  max_score: number;
  weight?: number;
  weight_pct?: number;
  status: string;
  actual_value?: string | number;
  value_label?: string;
  benchmark_rule?: string;
}

export interface TfScorecardReport {
  total_score: number;
  max_score: number;
  channel_level: string;
  revenue_sharing_eligible: boolean;
  revenue_share_tier: string;
  revenue_share_pct: number;
  pillars: TfScorecardPillar[];
}

export interface IMarketDataPort {
  getCandles(symbol: string, timeframe?: string, limit?: number): Promise<Candle[]>;
  getLatestPrice(symbol: string): Promise<number>;
  /** Optional live candle stream; historical-only adapters may omit it. */
  streamCandles?: (symbol: string, timeframe?: string) => AsyncIterable<Candle>;
  close?: () => void;
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

export * from './IUserPreferencesPort';
export * from './IReplayEnginePort';

