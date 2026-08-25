import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
import { type Candle, normalizeSymbol, getPipMultiplier } from '../../domain/models';

export interface ActiveExecutionTarget {
  id: string;
  symbol: string;
  action: string;
  isBuy: boolean;
  isPending: boolean;
  entryPrice: number;
  stopLoss: number;
  takeProfit: number;
  currentPrice?: number;
  openTime?: number;
}

/**
 * Strategy-Adaptive & Forward-Test Ready Active Position Visualizer (Interface-First Pattern).
 * Menampilkan garis harga interaktif (Entry, SL, TP) beserta badge sumbu harga
 * pada mode Bar Replay (simulasi in-flight) maupun Forward Testing (sinyal live).
 */
export class ActiveSignalOverlayLayer implements IChartLayer {
  public readonly id = 'signal-overlay';
  public readonly name = 'In-Flight & Active Signal Overlay (Entry/SL/TP)';
  public readonly shortLabel = 'Execution';
  public readonly description = 'Menampilkan garis batas level Entry, Stop Loss, dan Take Profit saat posisi sedang berjalan (In-Flight / Forward Test)';
  public readonly isUniversal = true;
  public visible = true;

  private priceLines: any[] = [];
  private currentSeries: any = null;
  private currentTargetId: string | null = null;

  isApplicableForStrategy(_strategyId: string, _category?: string): boolean {
    return true; // Universal layer: Wajib ada di seluruh strategi
  }

  private resolveActiveTarget(context: ChartLayerContext): ActiveExecutionTarget | null {
    const symbolNorm = normalizeSymbol(context.activeSymbol);

    // 1. Prioritas Utama: Forward Test / Live Signal aktif
    if (context.signal && normalizeSymbol(context.signal.symbol) === symbolNorm) {
      const sig = context.signal;
      const isBuy = sig.action.toUpperCase().includes('BUY');
      const isPending = sig.status === 'PENDING';
      return {
        id: sig.id || `live-signal-${sig.symbol}`,
        symbol: sig.symbol,
        action: sig.action,
        isBuy,
        isPending,
        entryPrice: sig.entryPrice,
        stopLoss: sig.stopLoss,
        takeProfit: sig.takeProfit1,
      };
    }

    // 2. Prioritas Kedua: In-Flight Simulated Trade saat Backtest / Bar Replay
    if (context.trades && context.trades.length > 0 && context.candles && context.candles.length > 0) {
      const latestCandle = context.candles[context.candles.length - 1];
      const latestCandleTime = latestCandle.time;

      const inFlightTrade = context.trades.find((t) =>
        normalizeSymbol(t.symbol) === symbolNorm &&
        t.open_time <= latestCandleTime &&
        (t.close_time == null || latestCandleTime < t.close_time)
      );

      if (inFlightTrade) {
        const isBuy = inFlightTrade.action.toUpperCase().includes('BUY');
        return {
          id: inFlightTrade.id,
          symbol: inFlightTrade.symbol,
          action: inFlightTrade.action,
          isBuy,
          isPending: false,
          entryPrice: inFlightTrade.open_price,
          stopLoss: inFlightTrade.stop_loss,
          takeProfit: inFlightTrade.take_profit,
          currentPrice: latestCandle.close,
          openTime: inFlightTrade.open_time,
        };
      }
    }

    return null;
  }

  render(context: ChartLayerContext): void {
    this.clear();
    if (!this.visible || !context || !context.candleSeries) return;

    const target = this.resolveActiveTarget(context);
    if (!target) {
      this.currentTargetId = null;
      return;
    }

    this.currentSeries = context.candleSeries;
    this.currentTargetId = target.id;

    try {
      const pipMult = getPipMultiplier(target.symbol);
      const slDist = Math.abs(target.entryPrice - target.stopLoss) * pipMult;
      const tpDist = Math.abs(target.takeProfit - target.entryPrice) * pipMult;
      const rr = slDist > 0 ? (tpDist / slDist) : 1.5;

      const actionText = target.action.replace(/[_/]/g, ' ').toUpperCase();



      // 1. Entry / Pending Order Line
      const entryTitle = target.isPending
        ? `⏳ PENDING ${actionText}`
        : `🔵 IN-FLIGHT ${actionText}`;

      const entryLine = context.candleSeries.createPriceLine({
        price: target.entryPrice,
        color: target.isPending ? '#f59e0b' : '#2962ff',
        lineWidth: 2,
        lineStyle: target.isPending ? 2 : 0, // Dashed for Pending, Solid for Filled
        axisLabelVisible: true,
        title: entryTitle,
      });
      this.priceLines.push(entryLine);

      // 2. Stop Loss (SL) Line
      const slLine = context.candleSeries.createPriceLine({
        price: target.stopLoss,
        color: '#f23645',
        lineWidth: 2,
        lineStyle: 2, // Dashed
        axisLabelVisible: true,
        title: `🛑 SL (-${slDist.toFixed(1)}p)`,
      });
      this.priceLines.push(slLine);

      // 3. Take Profit (TP) Line
      const tpLine = context.candleSeries.createPriceLine({
        price: target.takeProfit,
        color: '#089981',
        lineWidth: 2,
        lineStyle: 2, // Dashed
        axisLabelVisible: true,
        title: `🎯 TP (+${tpDist.toFixed(1)}p | 1:${rr.toFixed(1)})`,
      });
      this.priceLines.push(tpLine);
    } catch (e) {
      console.warn('[ActiveSignalOverlayLayer] createPriceLine warn:', e);
    }
  }

  /**
   * Ultra-Fast Replay / Forward Test Step Update (<0.01ms).
   */
  update(context: ChartLayerContext, _lastCandle: Candle): void {
    if (!this.visible || !context || !context.candleSeries) {
      if (this.priceLines.length > 0) this.clear();
      return;
    }

    const target = this.resolveActiveTarget(context);

    // Jika trade sudah selesai (TP/SL Hit) atau berganti target
    if (!target) {
      if (this.priceLines.length > 0) {
        this.clear();
      }
      this.currentTargetId = null;
      return;
    }

    // Jika target berganti (misal ada posisi baru masuk)
    if (this.currentTargetId !== target.id || this.priceLines.length !== 3) {
      this.render(context);
    }
  }

  clear(): void {
    if (this.currentSeries && this.priceLines.length > 0) {
      for (const line of this.priceLines) {
        try {
          this.currentSeries.removePriceLine(line);
        } catch (e) {}
      }
    }
    this.priceLines = [];
    this.currentTargetId = null;
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

