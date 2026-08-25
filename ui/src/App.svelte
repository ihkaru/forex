<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import TopBentoBar from './components/TopBentoBar.svelte';
  import LifecycleSidebar from './components/LifecycleSidebar.svelte';
  import TradingViewCanvas from './components/TradingViewCanvas.svelte';
  import SignalHud from './components/SignalHud.svelte';
  import InteractiveModal from './components/InteractiveModal.svelte';
  import StrategyTesterPanel from './components/tester/StrategyTesterPanel.svelte';

  import { AppCompositionRoot } from './index';
  import {
    type Candle,
    type Signal,
    type EdaReport,
    type ExecutionHudStatus,
    type PendingOrderState,
    type RunningPositionState,
    type SettledTradeState,
    type MarketScanContext,
    normalizeSymbol,
    getPipMultiplier,
    getValuedPipsMultiplier
  } from './domain/models';

  import type { StrategyDescriptor, MonteCarloReport, PointInTimeComplianceState } from './ports';
  import type { DetailedBacktestReport } from './ports/ITesterPort';

  import { TfComplianceGuard, TfPairSpec } from './domain/specs';
  import { Cpu, Dices, Search, Layers } from '@lucide/svelte';

  const composition = new AppCompositionRoot();

  const initialPrefs = composition.preferencesPort.loadPreferences();

  // Reactive State (Svelte 5 Runes)
  let activeNav = $state('terminal');
  let activeSymbol = $state(initialPrefs.activeSymbol || 'XAUUSD');
  let activePairs = $state<Array<{ symbol: string; base: string; quote: string; tier: number; multiplier: number }>>([
    { symbol: 'XAUUSD', base: 'XAU', quote: 'USD', tier: 4, multiplier: 0.5 },
    { symbol: 'USDCHF', base: 'USD', quote: 'CHF', tier: 1, multiplier: 2.0 },
    { symbol: 'GBPUSD', base: 'GBP', quote: 'USD', tier: 2, multiplier: 1.5 },
    { symbol: 'EURUSD', base: 'EUR', quote: 'USD', tier: 2, multiplier: 1.5 },
    { symbol: 'EURGBP', base: 'EUR', quote: 'GBP', tier: 1, multiplier: 2.0 },
    { symbol: 'NZDUSD', base: 'NZD', quote: 'USD', tier: 1, multiplier: 2.0 },
    { symbol: 'AUDUSD', base: 'AUD', quote: 'USD', tier: 1, multiplier: 2.0 },
  ]);

  let candles = $state<Candle[]>([]);
  let trades = $state<any[]>([]);
  let detailedBacktest = $state<DetailedBacktestReport | null>(null);
  let currentPrice = $state(0.85412);
  let activeSignal = $state<Signal | null>(null);

  // Execution HUD FSM State
  let hudStatus = $state<ExecutionHudStatus>('SCANNING');
  let pendingOrder = $state<PendingOrderState | null>(null);
  let runningPosition = $state<RunningPositionState | null>(null);
  let settledTrade = $state<SettledTradeState | null>(null);
  let scanContext = $state<MarketScanContext | null>(null);
  let lastReplayEvalTime = 0;


  // Multi-Strategy State

  let strategies = $state<StrategyDescriptor[]>([]);
  let selectedStrategyId = $state(initialPrefs.activeStrategyId || 'pola-n-v8');

  // Auto-persist active strategy & symbol on reactive change
  $effect(() => {
    composition.preferencesPort.savePreferences({
      activeStrategyId: selectedStrategyId,
      activeSymbol: activeSymbol,
    });
  });
  let selectedStrategy = $derived(
    strategies.find((s) => s.id === selectedStrategyId) || {
      id: 'pola-n-v8',
      name: 'TF Pola N Titan (v8 Quantum Leap All-Time Record Pro)',
      code: 'STRAT_POLA_N_V8_TITAN_PRO',
      description: 'Model kuantitatif rekor tertinggi Pola N Generasi 8 Titan khusus Emas (XAUUSD) dengan shallow impulse momentum window (0.15-0.85), buffer struktural 2.5 pips, target R:R kalibrasi 1:1.02 (+12,416.5 VP 10-Tahun, RF 12.18, PF 1.63, 1390 Trades).',
      category: 'GOLD_SPECIALIST',
      author: 'TF Quantitative Lab',
      winRatePct: 43.9,
      profitFactor: 1.63,
      recoveryFactor: 12.18,
      sharpeRatio: 4.65,
      sortinoRatio: 7.15,
      calmarRatio: 9.80,
      wferPct: 99.8,
      isTfCompliant: true,
      supportedSymbols: ['XAUUSD'],
      isSpecialist: true,
      specialistLabel: '👑 V8 TITAN PRO (+12,416 VP • RF 12.18 • PF 1.63)',
    }
  );






  // Monte Carlo State
  let monteCarloData = $state<MonteCarloReport | null>(null);

  let valuedPips = $state(3262.5);
  let currentMonthLabel = $state('Monthly Goal');
  let currentMonthVp = $state(0.0);
  let currentMonthTrades = $state(0);
  let targetPips = $state(300.0);
  let scorecardScore = $state(20);
  let scorecardTier = $state('MASTER_PRIORITY');
  let scorecardPillars = $state<any[]>([]);
  let wferPct = $state(94.8);
  let totalBars = $state(198534);
  let isTfQualified = $state(false);
  let complianceState = $state<PointInTimeComplianceState | null>(null);


  let isModalOpen = $state(false);
  let modalType = $state<'data-provenance' | 'lifecycle' | 'tf-hub' | 'wfa-lab' | 'eda' | 'monte-carlo' | 'multi-strategy'>('lifecycle');
  let edaReport = $state<EdaReport | null>(null);
  let backtestData = $state<any>(null);
  let scorecardData = $state<any>(null);

  async function loadConfig() {
    try {
      const res = await fetch('http://127.0.0.1:5000/api/config');
      if (res.ok) {
        const data = await res.json();
        if (data.active_pairs) {
          activePairs = data.active_pairs;
        }
      }
    } catch (e) {
      console.log('Using default pair config');
    }
  }

  async function loadStrategies() {
    try {
      strategies = await composition.strategyPort.getStrategies();
    } catch (e) {
      console.warn('Failed to load strategies:', e);
    }
  }

  async function loadMonteCarlo(symbol: string) {
    try {
      monteCarloData = await composition.monteCarloPort.getMonteCarloReport(symbol);
    } catch (e) {
      console.warn('Failed to load Monte Carlo report:', e);
    }
  }

  let streamAbortController: AbortController | null = null;

  function stopLiveStream() {
    if (streamAbortController) {
      streamAbortController.abort();
      streamAbortController = null;
    }
  }

  async function startLiveStream(symbol: string) {
    stopLiveStream();
    if (!composition.marketDataPort.streamCandles) return;

    const controller = new AbortController();
    streamAbortController = controller;

    try {
      const stream = composition.marketDataPort.streamCandles(symbol, 'M1');
      for await (const liveCandle of stream) {
        if (controller.signal.aborted) break;
        if (activeSymbol !== symbol) break;

        currentPrice = liveCandle.close;

        if (candles.length > 0) {
          const lastCandle = candles[candles.length - 1];
          if (liveCandle.time === lastCandle.time) {
            const updated: Candle = {
              ...lastCandle,
              high: Math.max(lastCandle.high, liveCandle.high),
              low: Math.min(lastCandle.low, liveCandle.low),
              close: liveCandle.close,
              volume: (lastCandle.volume ?? 0) + (liveCandle.volume ?? 0),
            };
            candles = [...candles.slice(0, -1), updated];
          } else if (liveCandle.time > lastCandle.time) {
            candles = [...candles, liveCandle];
          }
        }
      }
    } catch (e) {
      console.warn('Live stream closed or errored for', symbol, e);
    }
  }

  let selectedMarketSource = $state<'dukascopy' | 'mrg_demo' | 'mrg_real'>('dukascopy');

  async function loadMarketData(
    symbol: string,
    strategyId?: string,
    source?: 'dukascopy' | 'mrg_demo' | 'mrg_real'
  ) {
    activeSymbol = symbol;
    const stratId = strategyId || selectedStrategyId;
    const src = source || selectedMarketSource;

    try {
      const [candleData, tradeData, detailedData] = await Promise.all([
        composition.marketDataPort.getCandles(symbol, 'H1', 15000, src),
        composition.backtestPort.getTrades(symbol, stratId),
        composition.testerPort.getDetailedBacktestReport(symbol, stratId).catch(() => null),
      ]);
      if (candleData && candleData.length > 0) {
        candles = candleData;
        currentPrice = candleData[candleData.length - 1].close;
        evaluateExecutionState(symbol, candleData, false);
      }

      if (tradeData) {
        trades = tradeData;
        if (candles.length > 0) {
          evaluateExecutionState(symbol, candles, false);
        }
      }
      if (detailedData) {
        detailedBacktest = detailedData;
      }
      // Background fetch Monte Carlo
      loadMonteCarlo(symbol);

      // Start live websocket stream
      startLiveStream(symbol);
    } catch (e) {
      console.error('Failed to load market data for', symbol, e);
    }
  }

  function computeLastEma(candleList: Candle[], period: number): number {
    if (candleList.length === 0) return 0;
    if (candleList.length < period) return candleList[candleList.length - 1].close;
    // Windowed to the last 200 bars for O(1) instantaneous calculation on replay
    const list = candleList.length > 200 ? candleList.slice(candleList.length - 200) : candleList;
    const k = 2 / (period + 1);
    let ema = list[0].close;
    for (let i = 1; i < list.length; i++) {
      ema = list[i].close * k + ema * (1 - k);
    }
    return ema;
  }

  function computeLastRsi(candleList: Candle[], period: number = 14): number {
    if (candleList.length < period + 1) return 50;
    // Windowed to the last 150 bars for O(1) instantaneous calculation on replay
    const list = candleList.length > 150 ? candleList.slice(candleList.length - 150) : candleList;
    let gains = 0;
    let losses = 0;
    for (let i = 1; i <= period; i++) {
      const diff = list[i].close - list[i - 1].close;
      if (diff >= 0) gains += diff;
      else losses -= diff;
    }
    let avgGain = gains / period;
    let avgLoss = losses / period;

    for (let i = period + 1; i < list.length; i++) {
      const diff = list[i].close - list[i - 1].close;
      if (diff >= 0) {
        avgGain = (avgGain * (period - 1) + diff) / period;
        avgLoss = (avgLoss * (period - 1)) / period;
      } else {
        avgGain = (avgGain * (period - 1)) / period;
        avgLoss = (avgLoss * (period - 1) - diff) / period;
      }
    }
    if (avgLoss === 0) return 100;
    const rs = avgGain / avgLoss;
    return 100 - (100 / (1 + rs));
  }


  function evaluateExecutionState(symbol: string, candleList: Candle[], isReplay: boolean) {
    if (!candleList || candleList.length < 15) return;
    const last = candleList[candleList.length - 1];
    const currentBarTime = last.time;

    // 1. Dynamic Point-In-Time Metrics Calculation (Interface-First / Anti-Lookahead)
    if (trades && trades.length > 0) {
      const kpiResult = composition.replayKpiPort.computePointInTimeKpis(
        trades,
        currentBarTime,
        symbol,
        candleList.length
      );
      currentMonthLabel = kpiResult.currentMonthLabel;
      currentMonthVp = kpiResult.currentMonthVp;
      currentMonthTrades = kpiResult.currentMonthTrades;
      valuedPips = kpiResult.allTimeValuedPips;
      scorecardScore = kpiResult.scorecardScore;
      scorecardTier = kpiResult.scorecardTier;
      scorecardPillars = kpiResult.pillars;
      wferPct = kpiResult.wferPct;
      totalBars = kpiResult.verifiedBarsCount;
      complianceState = kpiResult.compliance;
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

    scanContext = {
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


    // 2. Check if a simulated trade is pending, running, or settled on this bar
    if (trades && trades.length > 0) {
      const symNorm = normalizeSymbol(symbol);
      const pipMult = getPipMultiplier(symbol);
      const vpMult = getValuedPipsMultiplier(symbol);

      // A. Pending Trade waiting for fill? (Posted at or before current bar, but not filled yet)
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

        pendingOrder = {
          tradeId: pendingTrade.id,
          action: pendingTrade.action.replace(/[_/]/g, ' '),
          entryPrice: openPrice,
          currentPrice: curPrice,
          stopLoss: pendingTrade.stop_loss,
          takeProfit: pendingTrade.take_profit,
          distancePips: distPips,
          postedTime: pendingTrade.posted_time,
          slaMinutes: 5,
        };
        runningPosition = null;
        settledTrade = null;
        activeSignal = null;
        hudStatus = 'PENDING';
        return;
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

        runningPosition = {
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
        };
        pendingOrder = null;
        settledTrade = null;
        activeSignal = null;
        hudStatus = 'RUNNING';
        return;
      }

      // C. Settled on this exact bar?
      const settledOnThisBar = trades.find(t => 
        normalizeSymbol(t.symbol) === symNorm &&
        t.close_time === currentBarTime
      );

      if (settledOnThisBar) {
        settledTrade = {
          tradeId: settledOnThisBar.id,
          action: settledOnThisBar.action.replace(/[_/]/g, ' '),
          openPrice: settledOnThisBar.open_price,
          closePrice: settledOnThisBar.close_price,
          pnlPips: settledOnThisBar.pnl_pips,
          valuedPips: settledOnThisBar.valued_pips,
          isWin: settledOnThisBar.is_win,
          exitReason: settledOnThisBar.exit_reason || (settledOnThisBar.is_win ? 'Take Profit Hit' : 'Stop Loss Hit')
        };
        pendingOrder = null;
        runningPosition = null;
        activeSignal = null;
        hudStatus = 'SETTLED';
        return;
      }
    }

    // 3. If no active position or pending or settled trade, status is SCANNING
    pendingOrder = null;
    runningPosition = null;
    settledTrade = null;
    activeSignal = null;
    hudStatus = 'SCANNING';
  }


  async function loadBacktestAndScorecard() {
    try {
      const [fullAudit, sc, bt] = await Promise.all([
        fetch('http://127.0.0.1:5000/api/audit/full').then(r => r.json()).catch(() => null),
        fetch('http://127.0.0.1:5000/api/scorecard').then(r => r.json()).catch(() => null),
        composition.backtestPort.runBacktest().catch(() => null),
      ]);

      if (fullAudit) {
        valuedPips = Number(fullAudit.total_portfolio_valued_pips || 0);
        currentMonthVp = Number(fullAudit.current_month_valued_pips || 0);
        currentMonthTrades = Number(fullAudit.current_month_trades || 0);
        targetPips = Number(fullAudit.monthly_tf_target_vp || 300.0);
        isTfQualified = Boolean(fullAudit.is_portfolio_tf_qualified);

        if (fullAudit.scorecard) {
          scorecardScore = Number(fullAudit.scorecard.total_score || 12);
          scorecardTier = fullAudit.scorecard.revenue_share_tier || 'SILVER_PRIORITY';
          scorecardPillars = fullAudit.scorecard.pillars || [];
        }

        if (fullAudit.walk_forward) {
          wferPct = Number(fullAudit.walk_forward.wfer_pct || 94.8);
          totalBars = Number(fullAudit.walk_forward.total_verified_bars || 198534);
        }
      } else if (bt) {
        backtestData = bt;
        valuedPips = bt.totalValuedPips;
        wferPct = bt.wferPct;
        isTfQualified = bt.isTfQualified;
      }

      if (sc) {
        scorecardData = sc;
        if (!fullAudit?.scorecard) {
          scorecardScore = sc.total_score;
          scorecardTier = sc.channel_level;
          scorecardPillars = sc.pillars;
        }
      }
    } catch (e) {
      console.warn('API sync warn:', e);
    }
  }

  let syncStatusMessage = $state<string | null>(null);

  async function handleSyncDelta() {
    try {
      syncStatusMessage = '⏳ Menghubungi node Dukascopy...';
      const report = await composition.deltaSyncPort.syncPairDelta(activeSymbol);
      syncStatusMessage = `⚡ Sync ${report.symbol}: ${report.message || '100% Up-to-Date'}`;
      await loadMarketData(activeSymbol);
      setTimeout(() => {
        syncStatusMessage = null;
      }, 4000);
    } catch (e) {
      syncStatusMessage = `⚠️ Gagal sync: ${e}`;
    }
  }

  function handleOpenProvenance() {
    modalType = 'data-provenance';
    isModalOpen = true;
  }

  async function handleOpenEda() {
    try {
      edaReport = await composition.edaPort.getEdaHealth(activeSymbol);
    } catch (e) {
      console.warn('Failed to load EDA report:', e);
    }
    modalType = 'eda';
    isModalOpen = true;
  }

  function handleNavClick(navId: string) {
    activeNav = navId;
    if (navId !== 'terminal') {
      modalType = navId as any;
      isModalOpen = true;
    }
  }

  onMount(async () => {
    await loadConfig();
    await loadStrategies();
    await loadMarketData(activeSymbol);
    await loadBacktestAndScorecard();
  });

  onDestroy(() => {
    stopLiveStream();
    composition.marketDataPort.close?.();
  });
</script>

<div class="min-h-screen bg-[#131722] text-[#d1d4dc] p-3 sm:p-5 flex flex-col gap-4 font-sans">
  <!-- Unified Top Master Bar Header -->
  <TopBentoBar
    {valuedPips}
    {currentMonthVp}
    {currentMonthTrades}
    {targetPips}
    {scorecardScore}
    {scorecardTier}
    {scorecardPillars}
    {wferPct}
    {totalBars}
    {isTfQualified}
    {strategies}
    {selectedStrategyId}
    onSelectStrategy={(stratId: string) => {
      selectedStrategyId = stratId;
      const strat = strategies.find((s) => s.id === stratId);
      let targetSym = activeSymbol;
      if (strat?.supportedSymbols && strat.supportedSymbols.length > 0 && !strat.supportedSymbols.includes(activeSymbol)) {
        targetSym = strat.supportedSymbols[0];
      }
      loadMarketData(targetSym, stratId);
    }}

    onOpenModelHub={() => handleNavClick('multi-strategy')}
    onOpenMonteCarlo={() => handleNavClick('monte-carlo')}
  />

  <!-- Main Terminal Layout: Slim Icon Sidebar + Main Canvas Grid -->
  <div class="flex flex-col lg:flex-row gap-4 items-stretch flex-1">
    <!-- Left Navigation & Slim Dock Sidebar -->
    <LifecycleSidebar
      {activeNav}
      onNavClick={handleNavClick}
    />

    <!-- Main Workspace Grid -->
    <main class="flex-1 flex flex-col gap-4 min-w-0">
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <!-- 2 Cols: TradingView Canvas with Strategy-Adaptive Layers -->
        <div class="lg:col-span-2 min-w-0">
          <TradingViewCanvas
            {activeSymbol}
            {activePairs}
            supportedSymbols={selectedStrategy?.supportedSymbols || []}
            isSpecialist={selectedStrategy?.isSpecialist || false}
            activeStrategyId={selectedStrategyId}
            activeStrategyCategory={selectedStrategy?.category}
            {strategies}
            {currentPrice}

            {candles}
            {trades}
            signal={activeSignal}
            {syncStatusMessage}
            selectedSource={selectedMarketSource}
            preferencesPort={composition.preferencesPort}
            onSelectSymbol={(sym) => loadMarketData(sym)}
            onSelectSource={(newSrc) => {
              selectedMarketSource = newSrc;
              loadMarketData(activeSymbol, selectedStrategyId, newSrc);
            }}
            onSyncDelta={handleSyncDelta}
            onOpenProvenance={handleOpenProvenance}

            onReplayChange={(displayed, isReplay, latestCandle) => {
              if (isReplay && displayed.length > 0) {
                currentPrice = latestCandle ? latestCandle.close : displayed[displayed.length - 1].close;
                const now = performance.now();
                if (now - lastReplayEvalTime > 80 || !latestCandle) {
                  lastReplayEvalTime = now;
                  evaluateExecutionState(activeSymbol, displayed, true);
                }
              } else if (candles.length > 0) {
                currentPrice = candles[candles.length - 1].close;
                evaluateExecutionState(activeSymbol, candles, false);
              }
            }}
          />

        </div>

        <!-- 1 Col: Intelligence & Execution Command Center (Aligned with Chart) -->
        <div class="lg:col-span-1 min-w-0">
          <SignalHud
            status={hudStatus}
            signal={activeSignal}
            {pendingOrder}
            {activeSymbol}
            activeStrategyName={selectedStrategy.name}
            {runningPosition}
            {settledTrade}
            {scanContext}
            {valuedPips}
            {currentMonthLabel}
            {currentMonthVp}
            {currentMonthTrades}
            {targetPips}
            {scorecardScore}
            {scorecardTier}
            {scorecardPillars}
            {wferPct}
            {totalBars}
            {complianceState}
            onScanSignal={() => loadMarketData(activeSymbol, selectedStrategyId)}
          />

        </div>

      </div>

      <!-- TradingView-Standard Strategy Tester Panel (Overview, Summary, Trades, Equity Curve) -->
      <StrategyTesterPanel
        report={detailedBacktest}
        {activeSymbol}
      />
    </main>
  </div>

  <!-- Interactive Data Analytics Modal -->
  <InteractiveModal
    isOpen={isModalOpen}
    {modalType}
    {activeSymbol}
    {selectedStrategyId}
    {edaReport}
    {backtestData}
    {scorecardData}
    {strategies}
    {monteCarloData}
    {candles}
    onSelectStrategy={(id) => {
      selectedStrategyId = id;
      const s = strategies.find(x => x.id === id);
      let targetSym = activeSymbol;
      if (s && s.supportedSymbols && s.supportedSymbols.length > 0 && !s.supportedSymbols.includes(activeSymbol)) {
        targetSym = s.supportedSymbols[0];
      }
      loadMarketData(targetSym, id);
    }}
    onClose={() => {
      isModalOpen = false;
      activeNav = 'terminal';
    }}
  />
</div>
