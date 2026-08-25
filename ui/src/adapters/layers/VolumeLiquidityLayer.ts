import { HistogramSeries, type ISeriesApi, type HistogramData } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import type { Candle } from '../../domain/models';

/**
 * Adapter Layer: VolumeLiquidityLayer (Interface-First Pattern)
 * Mengimplementasikan IChartLayer untuk memvisualisasikan volume transaksi Dukascopy ECN
 * dan mendeteksi lonjakan likuiditas institusional (Volume > 1.5x SMA 20).
 */
export class VolumeLiquidityLayer implements IChartLayer {
  public readonly id = 'volume-liquidity';
  public readonly name = 'Volume & Institutional Liquidity';
  public readonly shortLabel = 'Volume';
  public readonly description = 'Menampilkan histogram volume Dukascopy ECN dan mendeteksi lonjakan likuiditas besar (>1.5x SMA20)';
  public readonly isUniversal = true; // Universal layer: Tersedia untuk seluruh strategi
  public visible = true;

  private volumeSeries: ISeriesApi<'Histogram'> | null = null;

  isApplicableForStrategy(_strategyId: string, _category?: string): boolean {
    return true;
  }

  render(context: ChartLayerContext): void {
    if (!this.visible || !context.candles || context.candles.length === 0 || !context.chart) {
      this.clear();
      return;
    }

    try {
      if (!this.volumeSeries) {
        this.volumeSeries = context.chart.addSeries(HistogramSeries, {
          priceFormat: {
            type: 'volume',
          },
          priceScaleId: 'volume_scale',
        });

        context.chart.priceScale('volume_scale').applyOptions({
          scaleMargins: {
            top: 0.80, // Menyisakan 80% atas untuk candlestick
            bottom: 0,
          },
          visible: false,
        });
      }

      const formattedData = this.buildVolumeData(context.candles);
      this.volumeSeries.setData(formattedData);
    } catch (e) {
      console.warn('[VolumeLiquidityLayer] render error:', e);
    }
  }


  update(context: ChartLayerContext, lastCandle: Candle): void {
    if (!this.visible || !this.volumeSeries) return;

    const candles = context.candles;
    const len = candles.length;
    if (len === 0) return;

    // Hitung rata-rata volume 20 bar terakhir
    const lookback = Math.min(20, len);
    let sumVol = 0;
    for (let i = len - lookback; i < len; i++) {
      sumVol += candles[i].volume || 1.0;
    }
    const avgVol = sumVol / lookback;

    const vol = lastCandle.volume || 1.0;
    const isHighLiquidity = vol > avgVol * 1.5;
    const isUp = lastCandle.close >= lastCandle.open;

    // Skema warna dinamis likuiditas tinggi
    const color = isHighLiquidity
      ? (isUp ? '#00f2fe' : '#ff0055') // Neon Cyan/Pink untuk lonjakan likuiditas besar
      : (isUp ? '#08998166' : '#f2364566'); // Semi-transparan untuk volume normal

    this.volumeSeries.update({
      time: lastCandle.time as any,
      value: vol,
      color: color,
    });
  }

  clear(): void {
    if (this.volumeSeries) {
      try {
        this.volumeSeries.setData([]);
      } catch (e) {}
      this.volumeSeries = null;
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

  private buildVolumeData(candles: Candle[]): HistogramData[] {
    const len = candles.length;
    const data: HistogramData[] = [];

    // Hitung running average SMA 20 volume
    for (let i = 0; i < len; i++) {
      const c = candles[i];
      const start = Math.max(0, i - 19);
      let sum = 0;
      for (let j = start; j <= i; j++) {
        sum += candles[j].volume || 1.0;
      }
      const avgVol = sum / (i - start + 1);

      const vol = c.volume || 1.0;
      const isHighLiquidity = vol > avgVol * 1.5;
      const isUp = c.close >= c.open;

      const color = isHighLiquidity
        ? (isUp ? '#00f2fe' : '#ff0055') // High Liquidity Pulse
        : (isUp ? '#08998166' : '#f2364566'); // Normal Volume

      data.push({
        time: c.time as any,
        value: vol,
        color: color,
      });
    }

    return data;
  }
}
