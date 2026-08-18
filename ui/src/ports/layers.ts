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
}

export interface IChartLayer {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  visible: boolean;
  render(context: ChartLayerContext): void;
  clear(): void;
  toggle(context: ChartLayerContext): boolean;
}
