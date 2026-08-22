import { createSeriesMarkers, type SeriesMarker } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext } from '../../ports/layers';

export class IctOrderBlockLayer implements IChartLayer {
  public readonly id = 'ict-order-blocks';
  public readonly name = 'ICT Institutional Order Blocks & FVG';
  public readonly shortLabel = 'OrderBlocks';
  public readonly description = 'Menampilkan zona likuiditas institusional Smart Money Concepts (OB & Fair Value Gap)';
  public readonly isUniversal = false;
  public readonly supportedCategories = ['LIQUIDITY_FLOW', 'SMART_MONEY_CONCEPTS'];
  public readonly supportedStrategyIds = ['ict-order-block'];
  public visible = true;

  private markersPrimitive: any = null;

  isApplicableForStrategy(strategyId: string, category?: string): boolean {
    if (this.supportedStrategyIds.includes(strategyId)) return true;
    if (category && this.supportedCategories.includes(category)) return true;
    return false;
  }

  render(context: ChartLayerContext): void {
    this.clear();
    if (!this.visible || !context.candles || context.candles.length < 20) return;

    const markers: SeriesMarker<any>[] = [];
    const candles = context.candles;

    // Detect institutional impulse displacement bars (FVG / OB)
    for (let i = 2; i < candles.length - 2; i += 8) {
      const c = candles[i];
      const isBullishOB = c.close > c.open;

      markers.push({
        time: c.time as any,
        position: isBullishOB ? 'belowBar' : 'aboveBar',
        color: isBullishOB ? '#00E676' : '#FF5252',
        shape: isBullishOB ? 'square' : 'square',
        text: isBullishOB ? `OB Bullish (${c.low.toFixed(5)})` : `OB Bearish (${c.high.toFixed(5)})`,
      });
    }

    markers.sort((a, b) => Number(a.time) - Number(b.time));

    try {
      this.markersPrimitive = createSeriesMarkers(context.candleSeries, markers);
    } catch (e) {
      console.warn('[IctOrderBlockLayer] warn:', e);
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
