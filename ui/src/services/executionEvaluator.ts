import {
  type Candle,
  type ExecutionHudStatus,
  type MarketScanContext,
  type PendingOrderState,
  type RunningPositionState,
  type SettledTradeState,
  normalizeSymbol,
  getPipMultiplier,
  getValuedPipsMultiplier
} from '../domain/models';
import type { IReplayKpiPort } from '../ports/IReplayKpiPort';
import { computeLastEma, computeLastRsi } from './technicalIndicators';

export interface EvaluatedExecutionState {
  hudStatus: ExecutionHudStatus;
  pendingOrder: PendingOrderState | null;
  runningPosition: RunningPositionState | null;
  settledTrade: SettledTradeState | null;
  scanContext: MarketScanContext;
  kpi?: {
    currentMonthLabel: string;
    currentMonthVp: number;
    currentMonthTrades: number;
    valuedPips: number;
    scorecardScore: number;
    scorecardTier: string;
    scorecardPillars: any[];
    wferPct: number;
    totalBars: number;
    complianceState: any;
  };
}

export function evaluateExecutionState(
  symbol: string,
  candleList: Candle[],
  trades: any[],
  replayKpiPort?: IReplayKpiPort
): EvaluatedExecutionState | null {
  if (!candleList || candleList.length < 15) return null;
  const last = candleList[candleList.length - 1];
  const currentBarTime = last.time;

  let kpi: EvaluatedExecutionState['kpi'] = undefined;

  // 1. Dynamic Point-In-Time Metrics Calculation
  if (trades && trades.length > 0 && replayKpiPort) {
    const kpiResult = replayKpiPort.computePointInTimeKpis(
      trades,
      currentBarTime,
      symbol,
      candleList.length
    );
    kpi = {
      currentMonthLabel: kpiResult.currentMonthLabel,
      currentMonthVp: kpiResult.currentMonthVp,
      currentMonthTrades: kpiResult.currentMonthTrades,
      valuedPips: kpiResult.allTimeValuedPips,
      scorecardScore: kpiResult.scorecardScore,
      scorecardTier: kpiResult.scorecardTier,
      scorecardPillars: kpiResult.pillars,
      wferPct: kpiResult.wferPct,
      totalBars: kpiResult.verifiedBarsCount,
      complianceState: kpiResult.compliance,
    };
  }

  // 2. Compute Tactical Indicators
  const fastEma = computeLastEma(candleList, 12);
  const slowEma = computeLastEma(candleList, 36);
  const macroEma = computeLastEma(candleList, 100);
  const rsi = computeLastRsi(candleList, 14);

  const isBull = fastEma > slowEma && slowEma > macroEma;
  const isBear = fastEma < slowEma && slowEma < macroEma;
  const trend: 'BULLISH' | 'BEARISH' | 'NEUTRAL' = isBull ? 'BULLISH' : isBear ? 'BEARISH' : 'NEUTRAL';

  const d = new Date(currentBarTime * 1000);
  const utcHour = d.getUTCHours();
  const isLondonNy = utcHour >= 7 && utcHour <= 18;
  const sessionName = isLondonNy ? 'London / NY Active' : 'Asian / Off-Hours';

  const scanContext: MarketScanContext = {
    trend,
    fastEma,
    slowEma,
    rsi,
    sessionName,
    isSessionActive: isLondonNy,
    waitingReason: isBull
      ? 'Menunggu Retracement Pola N ke 61.8% Golden Pocket'
      : isBear
        ? 'Menunggu Pullback Bearish ke 61.8% Resistance'
        : 'Menunggu Formasi Trend Alignment'
  };

  // 3. Check if a simulated trade is pending, running, or settled on this bar
  if (trades && trades.length > 0) {
    const symNorm = normalizeSymbol(symbol);
    const pipMult = getPipMultiplier(symbol);
    const vpMult = getValuedPipsMultiplier(symbol);

    // A. Pending Trade waiting for fill?
    const pendingTrade = trades.find(t => {
      const pTime = t.posted_time ?? t.open_time;
      return normalizeSymbol(t.symbol) === symNorm &&
        pTime <= currentBarTime &&
        currentBarTime < t.open_time;
    });

    if (pendingTrade) {
      const openPrice = pendingTrade.open_price;
      const curPrice = last.close;
      const distPips = Math.abs(curPrice - openPrice) * pipMult;

      return {
        hudStatus: 'PENDING',
        pendingOrder: {
          tradeId: pendingTrade.id,
          action: pendingTrade.action.replace(/[_/]/g, ' '),
          entryPrice: openPrice,
          currentPrice: curPrice,
          stopLoss: pendingTrade.stop_loss,
          takeProfit: pendingTrade.take_profit,
          distancePips: distPips,
          postedTime: pendingTrade.posted_time,
          slaMinutes: 5,
        },
        runningPosition: null,
        settledTrade: null,
        scanContext,
        kpi
      };
    }

    // B. Running In-Flight Trade?
    const inFlightTrade = trades.find(t =>
      normalizeSymbol(t.symbol) === symNorm &&
      t.open_time <= currentBarTime &&
      (t.close_time == null || currentBarTime < t.close_time)
    );

    if (inFlightTrade) {
      const isTradeBuy = inFlightTrade.action.toUpperCase().includes('BUY');
      const openPrice = inFlightTrade.open_price;
      const curPrice = last.close;
      const floatingPips = isTradeBuy
        ? (curPrice - openPrice) * pipMult
        : (openPrice - curPrice) * pipMult;
      const floatingValuedPips = floatingPips * vpMult;

      const tpDist = Math.abs(inFlightTrade.take_profit - openPrice);
      const curDist = isTradeBuy ? (curPrice - openPrice) : (openPrice - curPrice);
      const progressPct = tpDist > 0 ? Math.round((curDist / tpDist) * 100) : 50;

      let openIdx = -1;
      for (let i = candleList.length - 1; i >= 0; i--) {
        if (candleList[i].time === inFlightTrade.open_time) {
          openIdx = i;
          break;
        }
      }
      const heldBars = openIdx >= 0 ? Math.max(1, candleList.length - 1 - openIdx + 1) : 1;

      return {
        hudStatus: 'RUNNING',
        pendingOrder: null,
        runningPosition: {
          tradeId: inFlightTrade.id,
          action: inFlightTrade.action.replace(/[_/]/g, ' '),
          openPrice,
          currentPrice: curPrice,
          stopLoss: inFlightTrade.stop_loss,
          takeProfit: inFlightTrade.take_profit,
          floatingPips,
          floatingValuedPips,
          progressToTpPct: progressPct,
          heldBarsCount: heldBars,
          isProfit: floatingPips >= 0
        },
        settledTrade: null,
        scanContext,
        kpi
      };
    }

    // C. Settled on this exact bar?
    const settledOnThisBar = trades.find(t =>
      normalizeSymbol(t.symbol) === symNorm &&
      t.close_time === currentBarTime
    );

    if (settledOnThisBar) {
      return {
        hudStatus: 'SETTLED',
        pendingOrder: null,
        runningPosition: null,
        settledTrade: {
          tradeId: settledOnThisBar.id,
          action: settledOnThisBar.action.replace(/[_/]/g, ' '),
          openPrice: settledOnThisBar.open_price,
          closePrice: settledOnThisBar.close_price,
          pnlPips: settledOnThisBar.pnl_pips,
          valuedPips: settledOnThisBar.valued_pips,
          isWin: settledOnThisBar.is_win,
          exitReason: settledOnThisBar.exit_reason || (settledOnThisBar.is_win ? 'Take Profit Hit' : 'Stop Loss Hit')
        },
        scanContext,
        kpi
      };
    }
  }

  // 4. Default: SCANNING
  return {
    hudStatus: 'SCANNING',
    pendingOrder: null,
    runningPosition: null,
    settledTrade: null,
    scanContext,
    kpi
  };
}
