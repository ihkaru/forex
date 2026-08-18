import type { IChartLayer, ChartLayerContext } from '../../ports/layers';

export class ActiveSignalOverlayLayer implements IChartLayer {
  public readonly id = 'signal-overlay';
  public readonly name = 'Active Signal R:R Overlay (Entry/SL/TP)';
  public readonly description = 'Menampilkan garis batas harga pesanan tunda, Stop Loss, dan Take Profit';
  public visible = true;

  private priceLines: any[] = [];

  render(context: ChartLayerContext): void {
    this.clear(context);
    if (!this.visible || !context.signal) return;

    const signal = context.signal;

    try {
      const entryLine = context.candleSeries.createPriceLine({
        price: signal.entryPrice,
        color: '#06b6d4',
        lineWidth: 2,
        lineStyle: 0,
        axisLabelVisible: true,
        title: `ENTRY (${signal.action})`,
      });
      const slLine = context.candleSeries.createPriceLine({
        price: signal.stopLoss,
        color: '#f43f5e',
        lineWidth: 2,
        lineStyle: 2,
        axisLabelVisible: true,
        title: 'STOP LOSS',
      });
      const tpLine = context.candleSeries.createPriceLine({
        price: signal.takeProfit1,
        color: '#10b981',
        lineWidth: 2,
        lineStyle: 0,
        axisLabelVisible: true,
        title: 'TAKE PROFIT 1',
      });

      this.priceLines.push({ series: context.candleSeries, line: entryLine });
      this.priceLines.push({ series: context.candleSeries, line: slLine });
      this.priceLines.push({ series: context.candleSeries, line: tpLine });
    } catch (e) {
      console.warn('[ActiveSignalOverlayLayer] warn:', e);
    }
  }

  clear(context?: ChartLayerContext): void {
    for (const pl of this.priceLines) {
      try {
        pl.series.removePriceLine(pl.line);
      } catch (e) {}
    }
    this.priceLines = [];
  }

  toggle(context: ChartLayerContext): boolean {
    this.visible = !this.visible;
    if (this.visible) {
      this.render(context);
    } else {
      this.clear(context);
    }
    return this.visible;
  }
}
