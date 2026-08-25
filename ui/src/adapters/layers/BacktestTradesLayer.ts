import { createSeriesMarkers, type SeriesMarker } from 'lightweight-charts';
import type { IChartLayer, ChartLayerContext, SimulatedTrade } from '../../ports/layers';
import { type Candle, normalizeSymbol, getPipMultiplier } from '../../domain/models';

export class BacktestTradesLayer implements IChartLayer {
  public readonly id = 'backtest-trades';
  public readonly name = 'Backtest Trades (PnL Markers & In-Flight Lines)';
  public readonly shortLabel = 'Trades';
  public readonly description = 'Menampilkan titik Entry, Exit Take Profit / Stop Loss, dan garis harga posisi aktif (In-Flight)';
  public readonly isUniversal = true;
  public visible = true;

  private markersPrimitive: any = null;
  private inFlightPriceLines: any[] = [];
  private currentCandleSeries: any = null;
  private currentInFlightId: string | null = null;

  isApplicableForStrategy(_strategyId: string, _category?: string): boolean {
    return true; // Universal layer: Wajib ada di seluruh strategi
  }

  private normSym(s?: string): string {
    return normalizeSymbol(s);
  }


  render(context: ChartLayerContext): void {
    if (!this.visible || !context.candleSeries) {
      this.clear();
      return;
    }

    this.currentCandleSeries = context.candleSeries;

    if (!context.trades || context.trades.length === 0) {
      this.clear();
      return;
    }

    const markers: SeriesMarker<any>[] = [];
    const activeSymbolNorm = this.normSym(context.activeSymbol);
    const sortedTrades = [...context.trades]
      .filter((t) => this.normSym(t.symbol) === activeSymbolNorm)
      .sort((a, b) => a.open_time - b.open_time);

    // Filter berdasarkan candle waktu aktif (Replay aware)
    const latestCandleTime = context.candles && context.candles.length > 0 
      ? context.candles[context.candles.length - 1].time 
      : Infinity;

    let activeTrade: { trade: SimulatedTrade; isPending: boolean } | null = null;

    for (const trade of sortedTrades) {
      const postedTime = trade.posted_time ?? trade.open_time;
      // Jangan tampilkan jika sinyal belum dibuat saat replay
      if (postedTime > latestCandleTime) continue;

      const isPending = trade.open_time > latestCandleTime;
      const isBuy = trade.action.toUpperCase().includes('BUY');
      const actionLabel = this.formatAction(trade.action);
      const openPriceStr = this.formatPrice(trade.open_price);

      if (isPending) {
        // Pending order marker di bar pembuat sinyal
        markers.push({
          time: postedTime as any,
          position: isBuy ? 'belowBar' : 'aboveBar',
          color: '#f59e0b',
          shape: isBuy ? 'arrowUp' : 'arrowDown',
          text: `⏳ PENDING ${actionLabel} @ ${openPriceStr}`,
        });
        activeTrade = { trade, isPending: true };
      } else {
        // Entry Marker pada saat Filled
        markers.push({
          time: trade.open_time as any,
          position: isBuy ? 'belowBar' : 'aboveBar',
          color: isBuy ? '#089981' : '#f23645',
          shape: isBuy ? 'arrowUp' : 'arrowDown',
          text: `${actionLabel} @ ${openPriceStr}`,
        });

        // Exit Marker (Hanya muncul jika trade sudah close pada atau sebelum waktu bar aktif)
        if (trade.close_time && trade.close_time <= latestCandleTime) {
          const isWin = trade.is_win;
          const vpSign = trade.valued_pips > 0 ? '+' : '';
          const vpText = `${vpSign}${trade.valued_pips.toFixed(1)} VP`;
          const exitLabel = isWin ? `🎯 TP HIT (${vpText})` : `🛑 SL HIT (${vpText})`;

          const exitPosition = isBuy 
            ? (isWin ? 'aboveBar' : 'belowBar') 
            : (isWin ? 'belowBar' : 'aboveBar');

          markers.push({
            time: trade.close_time as any,
            position: exitPosition,
            color: isWin ? '#089981' : '#f23645',
            shape: isWin ? 'circle' : 'square',
            text: exitLabel,
          });
        } else {
          // Trade ini sedang berjalan (In-Flight)
          activeTrade = { trade, isPending: false };
        }
      }
    }

    markers.sort((a, b) => Number(a.time) - Number(b.time));

    try {
      if (!this.markersPrimitive) {
        this.markersPrimitive = createSeriesMarkers(context.candleSeries, markers);
      } else {
        this.markersPrimitive.setMarkers(markers);
      }
    } catch (e) {
      console.warn('[BacktestTradesLayer] createSeriesMarkers warn:', e);
    }

    // Render Pending / In-Flight Price Lines
    this.renderPriceLines(context, activeTrade);
  }

  private renderPriceLines(context: ChartLayerContext, active: { trade: SimulatedTrade; isPending: boolean } | null): void {
    if (!active || !context.candleSeries) {
      this.clearPriceLines();
      return;
    }

    const { trade, isPending } = active;
    const cacheKey = `${trade.id}_${isPending ? 'pending' : 'running'}`;

    if (this.currentInFlightId === cacheKey && this.inFlightPriceLines.length === 3) {
      return; // Sudah terpasang untuk state ini
    }

    this.clearPriceLines();
    this.currentInFlightId = cacheKey;

    try {
      const pipMult = getPipMultiplier(trade.symbol);
      const slDist = Math.abs(trade.open_price - trade.stop_loss) * pipMult;
      const tpDist = Math.abs(trade.take_profit - trade.open_price) * pipMult;
      const rr = slDist > 0 ? (tpDist / slDist) : 1.5;
      const actionText = trade.action.replace(/[_/]/g, ' ').toUpperCase();

      // 1. Entry / Pending Order Line
      const entryLine = context.candleSeries.createPriceLine({
        price: trade.open_price,
        color: isPending ? '#f59e0b' : '#2962ff',
        lineWidth: 2,
        lineStyle: isPending ? 2 : 0, // Dashed if Pending, Solid if In-Flight
        axisLabelVisible: true,
        title: isPending ? `⏳ PENDING ${actionText}` : `🔵 IN-FLIGHT ${actionText}`,
      });
      this.inFlightPriceLines.push(entryLine);

      // 2. Stop Loss Line
      const slLine = context.candleSeries.createPriceLine({
        price: trade.stop_loss,
        color: '#f23645',
        lineWidth: 2,
        lineStyle: 2, // Dashed
        axisLabelVisible: true,
        title: `🛑 SL (-${slDist.toFixed(1)}p)`,
      });
      this.inFlightPriceLines.push(slLine);

      // 3. Take Profit Line
      const tpLine = context.candleSeries.createPriceLine({
        price: trade.take_profit,
        color: '#089981',
        lineWidth: 2,
        lineStyle: 2, // Dashed
        axisLabelVisible: true,
        title: `🎯 TP (+${tpDist.toFixed(1)}p | 1:${rr.toFixed(1)})`,
      });
      this.inFlightPriceLines.push(tpLine);
    } catch (e) {
      console.warn('[BacktestTradesLayer] createPriceLine error:', e);
    }
  }


  private clearPriceLines(): void {
    if (this.currentCandleSeries && this.inFlightPriceLines.length > 0) {
      for (const line of this.inFlightPriceLines) {
        try {
          this.currentCandleSeries.removePriceLine(line);
        } catch (e) {}
      }
    }
    this.inFlightPriceLines = [];
    this.currentInFlightId = null;
  }

  update(context: ChartLayerContext, _lastCandle: Candle): void {
    if (this.visible) {
      this.render(context);
    }
  }

  clear(): void {
    if (this.markersPrimitive) {
      try {
        this.markersPrimitive.setMarkers([]);
      } catch (e) {}
      this.markersPrimitive = null;
    }
    this.clearPriceLines();
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

  private formatAction(action: string): string {
    return action
      .replace(/_/g, ' ')
      .toLowerCase()
      .replace(/\b[a-z]/g, (c) => c.toUpperCase());
  }

  private formatPrice(price: number): string {
    return price > 500 ? price.toFixed(2) : price.toFixed(5);
  }
}


