import type { IChartLayer, ChartLayerContext } from '../../ports/layers';

export class ActiveSignalOverlayLayer implements IChartLayer {
  public readonly id = 'signal-overlay';
  public readonly name = 'Signal Execution Overlay (Entry/SL/TP)';
  public readonly shortLabel = 'Signal';
  public readonly description = 'Menampilkan garis batas level Entry, Stop Loss, dan Take Profit sinyal aktif';
  public readonly isUniversal = true;
  public visible = true;

  private priceLines: any[] = [];

  isApplicableForStrategy(_strategyId: string, _category?: string): boolean {
    return true; // Universal layer: Wajib ada di seluruh strategi
  }

  render(context: ChartLayerContext): void {
    this.clear();
    if (!this.visible || !context || !context.signal || !context.candleSeries) return;

    const sig = context.signal;

    try {
      // 1. Entry Line
      const entryLine = context.candleSeries.createPriceLine({
        price: sig.entryPrice,
        color: '#2962ff',
        lineWidth: 2,
        lineStyle: 0,
        axisLabelVisible: true,
        title: `ENTRY (${sig.action})`,
      });
      this.priceLines.push(entryLine);

      // 2. Stop Loss Line
      const slLine = context.candleSeries.createPriceLine({
        price: sig.stopLoss,
        color: '#f23645',
        lineWidth: 2,
        lineStyle: 2,
        axisLabelVisible: true,
        title: 'STOP LOSS (SL)',
      });
      this.priceLines.push(slLine);

      // 3. Take Profit 1 Line
      const tpLine = context.candleSeries.createPriceLine({
        price: sig.takeProfit1,
        color: '#089981',
        lineWidth: 2,
        lineStyle: 2,
        axisLabelVisible: true,
        title: 'TAKE PROFIT (TP)',
      });
      this.priceLines.push(tpLine);
    } catch (e) {
      console.warn('[ActiveSignalOverlayLayer] createPriceLine warn:', e);
    }
  }

  /**
   * Ultra-Fast Price Line Update (<0.01ms).
   * Menyesuaikan level Entry/SL/TP tanpa menghapus & membuat ulang objek PriceLine.
   */
  update(context: ChartLayerContext, _lastCandle: Candle): void {
    if (!this.visible || !context || !context.signal || !context.candleSeries) {
      if (this.priceLines.length > 0) this.clear();
      return;
    }

    if (this.priceLines.length === 3) {
      const sig = context.signal;
      try {
        this.priceLines[0].applyOptions({ price: sig.entryPrice, title: `ENTRY (${sig.action})` });
        this.priceLines[1].applyOptions({ price: sig.stopLoss });
        this.priceLines[2].applyOptions({ price: sig.takeProfit1 });
      } catch (e) {
        this.render(context);
      }
    } else {
      this.render(context);
    }
  }

  clear(): void {
    for (const line of this.priceLines) {
      try {
        line.applyOptions({ axisLabelVisible: false });
      } catch (e) {}
    }
    this.priceLines = [];
  }

  toggle(context: ChartLayerContext): boolean {
    this.visible = !this.visible;
    if (this.visible && context) {
      this.render(context);
    } else {
      this.clear();
    }
    return this.visible;
  }
}
