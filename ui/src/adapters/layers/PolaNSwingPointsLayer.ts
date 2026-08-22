import { createSeriesMarkers, type SeriesMarker } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import type { Candle } from '../../domain/models';

export class PolaNSwingPointsLayer implements IChartLayer {
  public readonly id = 'pola-n-swings';
  public readonly name = 'Pola N Fractal Swings (H1/L1/H2/L2)';
  public readonly shortLabel = 'Swings';
  public readonly description = 'Menandai titik swing geometri fraktal pembentuk struktur Pola N';
  public readonly isUniversal = false;
  public readonly supportedCategories = ['MARKET_STRUCTURE', 'GOLD_SPECIALIST'];
  public readonly supportedStrategyIds = ['pola-n-core', 'pola-n-v2'];
  public visible = true;

  private markersPrimitive: any = null;

  isApplicableForStrategy(strategyId: string, category?: string): boolean {
    if (this.supportedStrategyIds.includes(strategyId)) return true;
    if (category && this.supportedCategories.includes(category)) return true;
    return false;
  }

  render(context: ChartLayerContext): void {
    this.clear();
    if (!this.visible || !context.candles || context.candles.length < 15) return;

    const swings = this.detectSwings(context.candles, 5, 3);
    const markers: SeriesMarker<any>[] = [];

    for (let i = 0; i < swings.length; i++) {
      const s = swings[i];
      const label = s.isHigh ? (i % 2 === 0 ? 'H1' : 'H2') : (i % 2 === 0 ? 'L1' : 'L2');

      markers.push({
        time: s.time as any,
        position: s.isHigh ? 'aboveBar' : 'belowBar',
        color: s.isHigh ? '#06b6d4' : '#8b5cf6',
        shape: s.isHigh ? 'circle' : 'square',
        text: `${label} (${s.price.toFixed(5)})`,
      });
    }

    markers.sort((a, b) => (Number(a.time) - Number(b.time)));

    try {
      this.markersPrimitive = createSeriesMarkers(context.candleSeries, markers);
    } catch (e) {
      console.warn('[PolaNSwingPointsLayer] warn:', e);
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

  private detectSwings(
    candles: Candle[],
    leftBars: number = 5,
    rightBars: number = 3
  ): Array<{ time: number; price: number; isHigh: boolean }> {
    const swings: Array<{ time: number; price: number; isHigh: boolean }> = [];
    const len = candles.length;

    for (let i = leftBars; i < len - rightBars; i++) {
      const curr = candles[i];
      let isHigh = true;
      let isLow = true;

      for (let j = i - leftBars; j <= i + rightBars; j++) {
        if (j === i) continue;
        if (candles[j].high >= curr.high) isHigh = false;
        if (candles[j].low <= curr.low) isLow = false;
      }

      if (isHigh) {
        swings.push({ time: curr.time, price: curr.high, isHigh: true });
      } else if (isLow) {
        swings.push({ time: curr.time, price: curr.low, isHigh: false });
      }
    }

    return swings;
  }
}
