import type { IChartApi, ISeriesApi } from 'lightweight-charts';
import type { Candle, Signal } from '../domain/models';

export interface SimulatedTrade {
  id: string;
  symbol: string;
  action: string;
  open_time: number;
  open_price: number;
  close_time: number;
  close_price: number;
  stop_loss: number;
  take_profit: number;
  pnl_pips: number;
  valued_pips: number;
  is_win: boolean;
  exit_reason: string;
}

export interface ChartLayerContext {
  chart: IChartApi;
  candleSeries: ISeriesApi<'Candlestick'>;
  candles: Candle[];
  trades: SimulatedTrade[];
  signal: Signal | null;
  activeSymbol: string;
  activeStrategyId?: string;
  activeStrategyCategory?: string;
}

/**
 * Interface-First Strategy-Adaptive Chart Layer Contract.
 * Memisahkan layer universal (wajib di semua strategi) vs layer khusus strategi.
 */
export interface IChartLayer {
  readonly id: string;
  readonly name: string;
  readonly shortLabel: string;
  readonly description: string;
  readonly isUniversal: boolean; // True jika wajib ada di seluruh strategi (Trades, Signal R:R)
  readonly supportedCategories?: string[];
  readonly supportedStrategyIds?: string[];
  visible: boolean;

  /**
   * Mengevaluasi apakah layer ini relevan untuk strategi yang aktif saat ini.
   */
  isApplicableForStrategy(strategyId: string, category?: string): boolean;

  render(context: ChartLayerContext): void;
  update?(context: ChartLayerContext, lastCandle: Candle): void;
  clear(): void;
  toggle(context: ChartLayerContext): boolean;
}
