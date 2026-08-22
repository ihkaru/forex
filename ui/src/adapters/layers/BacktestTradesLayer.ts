import { createSeriesMarkers, type SeriesMarker } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext } from '../../ports/layers';

export class BacktestTradesLayer implements IChartLayer {
  public readonly id = 'backtest-trades';
  public readonly name = 'Backtest Trades (PnL Markers)';
  public readonly shortLabel = 'Trades';
  public readonly description = 'Menampilkan titik Entry & Exit Take Profit / Stop Loss dari hasil simulasi deterministik';
  public readonly isUniversal = true;
  public visible = true;

  private markersPrimitive: any = null;

  isApplicableForStrategy(_strategyId: string, _category?: string): boolean {
    return true; // Universal layer: Wajib ada di seluruh strategi
  }

  render(context: ChartLayerContext): void {
    this.clear();
    if (!this.visible || !context.trades || context.trades.length === 0) return;

    const markers: SeriesMarker<any>[] = [];
    const sortedTrades = [...context.trades].sort((a, b) => a.open_time - b.open_time);

    for (const trade of sortedTrades) {
      const isBuy = trade.action.includes('Buy') || trade.action.includes('BUY');
      
      markers.push({
        time: trade.open_time as any,
        position: isBuy ? 'belowBar' : 'aboveBar',
        color: trade.is_win ? '#10b981' : '#f43f5e',
        shape: isBuy ? 'arrowUp' : 'arrowDown',
        text: `${trade.action.replace('_LIMIT', '')} @ ${trade.open_price.toFixed(5)} (${trade.is_win ? '+' : ''}${trade.valued_pips.toFixed(1)} VP)`,
      });

      if (trade.close_time && trade.close_time > trade.open_time) {
        markers.push({
          time: trade.close_time as any,
          position: isBuy ? 'aboveBar' : 'belowBar',
          color: trade.is_win ? '#10b981' : '#f43f5e',
          shape: 'circle',
          text: trade.is_win ? '🎯 TP HIT' : '🛑 SL HIT',
        });
      }
    }

    markers.sort((a, b) => (Number(a.time) - Number(b.time)));

    try {
      this.markersPrimitive = createSeriesMarkers(context.candleSeries, markers);
    } catch (e) {
      console.warn('[BacktestTradesLayer] createSeriesMarkers warn:', e);
    }
  }

  clear(): void {
    if (this.markersPrimitive) {
      try {
        this.markersPrimitive.setMarkers([]);
      } catch (e) {}
      this.markersPrimitive = null;
    }
  }

  toggle(context: ChartLayerContext): boolean {
    this.visible = !this.visible;
    if (this.visible) {
      this.render(context);
    } else {
      this.clear();
    }
    return this.visible;
  }
}
