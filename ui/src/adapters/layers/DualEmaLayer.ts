import { LineSeries, type ISeriesApi, type LineData } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import type { Candle } from '../../domain/models';

export class DualEmaLayer implements IChartLayer {
  public readonly id = 'dual-ema';
  public readonly name = 'Dual EMA Trend (20 / 50)';
  public readonly description = 'Indikator tren Exponential Moving Average periode 20 (Cyan) dan 50 (Amber)';
  public visible = true;

  private ema20Series: ISeriesApi<'Line'> | null = null;
  private ema50Series: ISeriesApi<'Line'> | null = null;

  render(context: ChartLayerContext): void {
    if (!this.visible) {
      this.clear();
      return;
    }
    if (!context.candles || context.candles.length === 0) return;

    if (!this.ema20Series) {
      this.ema20Series = context.chart.addSeries(LineSeries, {
        color: '#06b6d4',
        lineWidth: 2,
        title: 'EMA 20',
        priceLineVisible: false,
        lastValueVisible: true,
      });
    }

    if (!this.ema50Series) {
      this.ema50Series = context.chart.addSeries(LineSeries, {
        color: '#f59e0b',
        lineWidth: 2,
        title: 'EMA 50',
        priceLineVisible: false,
        lastValueVisible: true,
      });
    }

    const ema20 = this.computeEma(context.candles, 20);
    const ema50 = this.computeEma(context.candles, 50);

    this.ema20Series.setData(ema20);
    this.ema50Series.setData(ema50);
  }

  clear(): void {
    if (this.ema20Series) {
      this.ema20Series.setData([]);
    }
    if (this.ema50Series) {
      this.ema50Series.setData([]);
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

  private computeEma(data: Candle[], period: number): LineData[] {
    if (data.length < period) return [];
    const k = 2 / (period + 1);
    const emaList: LineData[] = [];
    let ema = data[0].close;

    for (let i = 0; i < data.length; i++) {
      if (i === 0) {
        ema = data[i].close;
      } else {
        ema = data[i].close * k + ema * (1 - k);
      }
      emaList.push({ time: data[i].time as any, value: ema });
    }
    return emaList;
  }
}
