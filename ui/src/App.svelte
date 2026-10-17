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
      const res = await fetch(`${composition.baseUrl}/config`);
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
              open: liveCandle.open ?? lastCandle.open,
              high: Math.max(lastCandle.high, liveCandle.high),
              low: Math.min(lastCandle.low, liveCandle.low),
              close: liveCandle.close,
              volume: liveCandle.volume ?? lastCandle.volume,
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

  let selectedMarketSource = $state<'dukascopy' | 'mrg_demo' | 'mrg_real'>('mrg_real');

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
        composition.backtestPort.getTrades(symbol, stratId, src),
        composition.testerPort.getDetailedBacktestReport(symbol, stratId, src).catch(() => null),
      ]);
      if (candleData) {
        candles = candleData;
        if (candleData.length > 0) {
          currentPrice = candleData[candleData.length - 1].close;
          evaluateExecutionState(symbol, candleData, false);
        }
      }

      if (tradeData) {
        trades = tradeData;
        if (candles.length > 0) {
          evaluateExecutionState(symbol, candles, false);
        }
      } else {
        trades = [];
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

  import { evaluateExecutionState as evaluateMarketExecutionState } from './services/executionEvaluator';

  function evaluateExecutionState(symbol: string, candleList: Candle[], _isReplay: boolean) {
    const res = evaluateMarketExecutionState(symbol, candleList, trades, composition.replayKpiPort);
    if (!res) return;
    hudStatus = res.hudStatus;
    pendingOrder = res.pendingOrder;
    runningPosition = res.runningPosition;
    settledTrade = res.settledTrade;
    scanContext = res.scanContext;
    activeSignal = null;
    if (res.kpi) {
      currentMonthLabel = res.kpi.currentMonthLabel;
      currentMonthVp = res.kpi.currentMonthVp;
      currentMonthTrades = res.kpi.currentMonthTrades;
      valuedPips = res.kpi.valuedPips;
      scorecardScore = res.kpi.scorecardScore;
      scorecardTier = res.kpi.scorecardTier;
      scorecardPillars = res.kpi.scorecardPillars;
      wferPct = res.kpi.wferPct;
      totalBars = res.kpi.totalBars;
      complianceState = res.kpi.complianceState;
    }
  }


  async function loadBacktestAndScorecard() {
    try {
      const [fullAudit, sc, bt] = await Promise.all([
        fetch(`${composition.baseUrl}/audit/full`).then(r => r.json()).catch(() => null),
        fetch(`${composition.baseUrl}/scorecard`).then(r => r.json()).catch(() => null),
        composition.backtestPort.runBacktest(selectedStrategyId, selectedMarketSource).catch(() => null),
      ]);

      if (bt) {
        backtestData = bt;
        valuedPips = bt.totalValuedPips;
        wferPct = bt.wferPct || 94.8;
        isTfQualified = bt.isTfQualified;
      } else if (fullAudit) {
        valuedPips = Number(fullAudit.total_portfolio_valued_pips || 0);
        isTfQualified = Boolean(fullAudit.is_portfolio_tf_qualified);
      }

      if (fullAudit) {
        currentMonthVp = Number(fullAudit.current_month_valued_pips || 0);
        currentMonthTrades = Number(fullAudit.current_month_trades || 0);
        targetPips = Number(fullAudit.monthly_tf_target_vp || 300.0);

        if (fullAudit.scorecard) {
          scorecardScore = Number(fullAudit.scorecard.total_score || 12);
          scorecardTier = fullAudit.scorecard.revenue_share_tier || 'SILVER_PRIORITY';
          scorecardPillars = fullAudit.scorecard.pillars || [];
        }

        if (fullAudit.walk_forward && !bt) {
          wferPct = Number(fullAudit.walk_forward.wfer_pct || 94.8);
          totalBars = Number(fullAudit.walk_forward.total_verified_bars || 198534);
        }
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
      loadBacktestAndScorecard();
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
              loadBacktestAndScorecard();
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
      loadBacktestAndScorecard();
    }}
    onClose={() => {
      isModalOpen = false;
      activeNav = 'terminal';
    }}
  />
</div>
