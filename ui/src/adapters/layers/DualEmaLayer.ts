import { LineSeries, type ISeriesApi } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import type { Candle } from '../../domain/models';

export class DualEmaLayer implements IChartLayer {
  public readonly id = 'dual-ema';
  public readonly name = 'Dual EMA Trend (EMA 20 / EMA 50)';
  public readonly shortLabel = 'EMA';
  public readonly description = 'Menampilkan kurva tren Fast EMA 20 (Cyan) dan Slow EMA 50 (Orange)';
  public readonly isUniversal = false;
  public readonly supportedCategories = ['MARKET_STRUCTURE', 'GOLD_SPECIALIST', 'TREND_FOLLOWING'];
  public readonly supportedStrategyIds = ['pola-n-core', 'pola-n-v2', 'dual-ema-trend'];
  public visible = true;

  private emaFastSeries: ISeriesApi<'Line'> | null = null;
  private emaSlowSeries: ISeriesApi<'Line'> | null = null;
  private chartRef: any = null;

  isApplicableForStrategy(strategyId: string, category?: string): boolean {
    if (this.supportedStrategyIds.includes(strategyId)) return true;
    if (category && this.supportedCategories.includes(category)) return true;
    return false;
  }

  render(context: ChartLayerContext): void {
    this.clear();
    if (!this.visible || !context || !context.chart || !context.candles || context.candles.length < 50) return;

    this.chartRef = context.chart;

    try {
      const emaFastData = this.calculateEmaData(context.candles, 20);
      const emaSlowData = this.calculateEmaData(context.candles, 50);

      this.emaFastSeries = context.chart.addSeries(LineSeries, {
        color: '#2962ff',
        lineWidth: 2,
        title: 'EMA 20',
        priceLineVisible: false,
        lastValueVisible: true,
      });
      this.emaFastSeries.setData(emaFastData as any);

      this.emaSlowSeries = context.chart.addSeries(LineSeries, {
        color: '#f5c344',
        lineWidth: 2,
        title: 'EMA 50',
        priceLineVisible: false,
        lastValueVisible: true,
      });
      this.emaSlowSeries.setData(emaSlowData as any);
    } catch (e) {
      console.warn('[DualEmaLayer] addSeries(LineSeries) warn:', e);
    }
  }

  clear(): void {
    if (this.chartRef) {
      if (this.emaFastSeries) {
        try {
          this.chartRef.removeSeries(this.emaFastSeries);
        } catch (e) {}
        this.emaFastSeries = null;
      }
      if (this.emaSlowSeries) {
        try {
          this.chartRef.removeSeries(this.emaSlowSeries);
        } catch (e) {}
        this.emaSlowSeries = null;
      }
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

  private calculateEmaData(candles: Candle[], period: number): Array<{ time: number; value: number }> {
    const k = 2 / (period + 1);
    let ema = candles[0].close;
    const result: Array<{ time: number; value: number }> = [];

    for (let i = 0; i < candles.length; i++) {
      const c = candles[i];
      if (i === 0) {
        ema = c.close;
      } else {
        ema = c.close * k + ema * (1 - k);
      }
      if (i >= period - 1) {
        result.push({ time: c.time, value: ema });
      }
    }

    return result;
  }
}
