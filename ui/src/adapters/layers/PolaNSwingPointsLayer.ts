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
  private cachedMarkers: SeriesMarker<any>[] = [];

  isApplicableForStrategy(strategyId: string, category?: string): boolean {
    if (this.supportedStrategyIds.includes(strategyId)) return true;
    if (category && this.supportedCategories.includes(category)) return true;
    return false;
  }

  render(context: ChartLayerContext): void {
    if (!this.visible || !context.candles || context.candles.length < 15 || !context.candleSeries) {
      this.clear();
      return;
    }

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
    this.cachedMarkers = markers;

    try {
      if (!this.markersPrimitive) {
        this.markersPrimitive = createSeriesMarkers(context.candleSeries, markers);
      } else {
        this.markersPrimitive.setMarkers(markers);
      }
    } catch (e) {
      console.warn('[PolaNSwingPointsLayer] warn:', e);
    }
  }

  /**
   * Ultra-Fast Incremental Swing Check (<0.02ms).
   * Hanya memeriksa apakah candle pada indeks (len - 4) terkonfirmasi sebagai swing baru.
   */
  update(context: ChartLayerContext, _lastCandle: Candle): void {
    if (!this.visible || !context.candles || context.candles.length < 15 || !context.candleSeries) return;

    const candles = context.candles;
    const len = candles.length;
    const i = len - 4; // Candle yang baru saja memenuhi rightBars = 3

    if (i < 5) return;

    const curr = candles[i];
    let isHigh = true;
    let isLow = true;

    for (let j = i - 5; j <= i + 3; j++) {
      if (j === i) continue;
      if (candles[j].high >= curr.high) isHigh = false;
      if (candles[j].low <= curr.low) isLow = false;
    }

    if (isHigh || isLow) {
      const label = isHigh ? (this.cachedMarkers.length % 2 === 0 ? 'H1' : 'H2') : (this.cachedMarkers.length % 2 === 0 ? 'L1' : 'L2');
      const newMarker: SeriesMarker<any> = {
        time: curr.time as any,
        position: isHigh ? 'aboveBar' : 'belowBar',
        color: isHigh ? '#06b6d4' : '#8b5cf6',
        shape: isHigh ? 'circle' : 'square',
        text: `${label} (${(isHigh ? curr.high : curr.low).toFixed(5)})`,
      };
      this.cachedMarkers.push(newMarker);
      this.cachedMarkers.sort((a, b) => Number(a.time) - Number(b.time));

      if (this.markersPrimitive) {
        try {
          this.markersPrimitive.setMarkers(this.cachedMarkers);
        } catch (e) {}
      } else {
        this.render(context);
      }
    }
  }

  clear(): void {
    if (this.markersPrimitive) {
      try {
        this.markersPrimitive.setMarkers([]);
      } catch (e) {}
      this.markersPrimitive = null;
    }
    this.cachedMarkers = [];
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
