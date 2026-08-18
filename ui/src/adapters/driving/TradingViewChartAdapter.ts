import { createChart, CandlestickSeries, LineSeries } from 'lightweight-charts';
import type { Candle, Signal } from '../../domain/models';
import type { IMarketDataPort } from '../../ports';

export class TradingViewChartAdapter {
  private chart: any = null;
  private candleSeries: any = null;
  private ema20Series: any = null;
  private ema50Series: any = null;
  private priceLines: any[] = [];

  constructor(
    private readonly containerId: string,
    private readonly marketDataPort: IMarketDataPort
  ) {}

  init(): void {
    const container = document.getElementById(this.containerId);
    if (!container) return;

    this.chart = createChart(container, {
      width: container.clientWidth,
      height: 480,
      layout: {
        background: { color: '#0b0f17' },
        textColor: '#94a3b8',
        fontFamily: "'JetBrains Mono', monospace",
        fontSize: 11,
      },
      grid: {
        vertLines: { color: 'rgba(255, 255, 255, 0.04)' },
        horzLines: { color: 'rgba(255, 255, 255, 0.04)' },
      },
      crosshair: {
        mode: 1,
        vertLine: { color: '#06b6d4', width: 1, style: 2 },
        horzLine: { color: '#06b6d4', width: 1, style: 2 },
      },
      rightPriceScale: {
        borderColor: 'rgba(255, 255, 255, 0.08)',
        scaleMargins: { top: 0.15, bottom: 0.2 },
        autoScale: true,
      },
      timeScale: {
        borderColor: 'rgba(255, 255, 255, 0.08)',
        timeVisible: true,
        secondsVisible: false,
      },
    });

    this.candleSeries = this.chart.addSeries(CandlestickSeries, {
      upColor: '#10b981',
      downColor: '#f43f5e',
      borderVisible: false,
      wickUpColor: '#10b981',
      wickDownColor: '#f43f5e',
    });

    this.ema20Series = this.chart.addSeries(LineSeries, {
      color: '#06b6d4',
      lineWidth: 2,
      title: 'EMA 20',
    });

    this.ema50Series = this.chart.addSeries(LineSeries, {
      color: '#f59e0b',
      lineWidth: 2,
      title: 'EMA 50',
    });

    window.addEventListener('resize', () => {
      if (this.chart && container) {
        this.chart.applyOptions({ width: container.clientWidth });
      }
    });
  }

  async renderSymbol(symbol: string): Promise<void> {
    const candles = await this.marketDataPort.getCandles(symbol);
    if (!candles || candles.length === 0) return;

    this.candleSeries.setData(candles);

    // Compute EMA 20 & 50 via functional pipeline (Pure Composition)
    const ema20 = this.computeEma(candles, 20);
    const ema50 = this.computeEma(candles, 50);
    this.ema20Series.setData(ema20);
    this.ema50Series.setData(ema50);

    this.chart.timeScale().fitContent();
  }

  drawSignalOverlays(signal: Signal): void {
    this.priceLines.forEach((pl) => this.candleSeries.removePriceLine(pl));
    this.priceLines = [];

    const entryLine = this.candleSeries.createPriceLine({
      price: signal.entryPrice,
      color: '#06b6d4',
      lineWidth: 2,
      lineStyle: 0,
      axisLabelVisible: true,
      title: `ENTRY (${signal.action})`,
    });
    this.priceLines.push(entryLine);

    const slLine = this.candleSeries.createPriceLine({
      price: signal.stopLoss,
      color: '#f43f5e',
      lineWidth: 2,
      lineStyle: 2,
      axisLabelVisible: true,
      title: 'STOP LOSS',
    });
    this.priceLines.push(slLine);

    const tpLine = this.candleSeries.createPriceLine({
      price: signal.takeProfit1,
      color: '#10b981',
      lineWidth: 2,
      lineStyle: 0,
      axisLabelVisible: true,
      title: 'TAKE PROFIT 1',
    });
    this.priceLines.push(tpLine);
  }

  private computeEma(candles: Candle[], period: number): Array<{ time: number; value: number }> {
    if (candles.length < period) return [];
    const k = 2 / (period + 1);
    const emaList: Array<{ time: number; value: number }> = [];
    let ema = candles[0].close;

    for (let i = 0; i < candles.length; i++) {
      if (i === 0) {
        ema = candles[i].close;
      } else {
        ema = candles[i].close * k + ema * (1 - k);
      }
      emaList.push({ time: candles[i].time, value: ema });
    }
    return emaList;
  }
}
