import { createSeriesMarkers, type SeriesMarker } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import type { Candle } from '../../domain/models';

export class PolaNSwingPointsLayer implements IChartLayer {
  public readonly id = 'pola-n-swings';
  public readonly name = 'Pola N Fractal Swings (H1/L1/H2/L2)';
  public readonly description = 'Menandai titik swing geometri fraktal pembentuk struktur Pola N';
  public visible = true;

  private markersPrimitive: any = null;

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

  private detectSwings(candles: Candle[], leftBars = 5, rightBars = 3) {
    const swings: Array<{ time: number; price: number; isHigh: boolean }> = [];
    const total = candles.length;
    if (total < leftBars + rightBars + 1) return swings;

    // Ambil 100 bar terakhir untuk efisiensi marker visual
    const startIdx = Math.max(leftBars, total - 120);

    for (let i = startIdx; i < total - rightBars; i++) {
      const curHigh = candles[i].high;
      const curLow = candles[i].low;

      let isHigh = true;
      for (let l = 1; l <= leftBars; l++) if (candles[i - l].high >= curHigh) isHigh = false;
      for (let r = 1; r <= rightBars; r++) if (candles[i + r].high >= curHigh) isHigh = false;

      if (isHigh) {
        swings.push({ time: candles[i].time, price: curHigh, isHigh: true });
        continue;
      }

      let isLow = true;
      for (let l = 1; l <= leftBars; l++) if (candles[i - l].low <= curLow) isLow = false;
      for (let r = 1; r <= rightBars; r++) if (candles[i + r].low <= curLow) isLow = false;

      if (isLow) {
        swings.push({ time: candles[i].time, price: curLow, isHigh: false });
      }
    }
    return swings;
  }
}
