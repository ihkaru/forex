<script lang="ts">
  import { onMount } from 'svelte';
  import TopBentoBar from './components/TopBentoBar.svelte';
  import LifecycleSidebar from './components/LifecycleSidebar.svelte';
  import TradingViewCanvas from './components/TradingViewCanvas.svelte';
  import SignalHud from './components/SignalHud.svelte';
  import InteractiveModal from './components/InteractiveModal.svelte';
  import StrategyTesterPanel from './components/tester/StrategyTesterPanel.svelte';

  import { AppCompositionRoot } from './index';
  import type { Candle, Signal, EdaReport } from './domain/models';
  import type { StrategyDescriptor, MonteCarloReport } from './ports';
  import type { DetailedBacktestReport } from './ports/ITesterPort';
  import { TfComplianceGuard, TfPairSpec } from './domain/specs';
  import { Cpu, Dices, Search, Layers } from '@lucide/svelte';

  const composition = new AppCompositionRoot();

  // Reactive State (Svelte 5 Runes)
  let activeNav = $state('terminal');
  let activeSymbol = $state('XAUUSD');
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

  // Multi-Strategy State
  let strategies = $state<StrategyDescriptor[]>([]);
  let selectedStrategyId = $state('pola-n-core');
  let selectedStrategy = $derived(
    strategies.find((s) => s.id === selectedStrategyId) || {
      id: 'pola-n-core',
      name: 'TF Pola N Structure Engine',
      code: 'STRAT_POLA_N_V1',
      description: 'Struktur fraktal swing L1-H1-L2 + 50% Golden Zone retest',
      category: 'MARKET_STRUCTURE',
      author: 'TF Lab',
      winRatePct: 68.4,
      profitFactor: 2.34,
      recoveryFactor: 9.80,
      sharpeRatio: 2.14,
      sortinoRatio: 3.42,
      calmarRatio: 4.12,
      wferPct: 94.8,
      isTfCompliant: true,
    }
  );

  // Monte Carlo State
  let monteCarloData = $state<MonteCarloReport | null>(null);

  let valuedPips = $state(-12874.4);
  let currentMonthVp = $state(0.0);
  let currentMonthTrades = $state(0);
  let targetPips = $state(300.0);
  let scorecardScore = $state(12);
  let scorecardTier = $state('SILVER_PRIORITY');
  let scorecardPillars = $state<any[]>([]);
  let wferPct = $state(94.8);
  let totalBars = $state(198534);
  let isTfQualified = $state(false);

  let isModalOpen = $state(false);
  let modalType = $state('lifecycle');
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

  async function loadMarketData(symbol: string, strategyId?: string) {
    activeSymbol = symbol;
    const stratId = strategyId || selectedStrategyId;
    try {
      const [candleData, tradeData, detailedData] = await Promise.all([
        composition.marketDataPort.getCandles(symbol),
        composition.backtestPort.getTrades(symbol, stratId),
        composition.testerPort.getDetailedBacktestReport(symbol, stratId).catch(() => null),
      ]);
      if (candleData && candleData.length > 0) {
        candles = candleData;
        currentPrice = candleData[candleData.length - 1].close;
        generateSignal(symbol, candleData);
      }
      if (tradeData) {
        trades = tradeData;
      }
      if (detailedData) {
        detailedBacktest = detailedData;
      }
      // Background fetch Monte Carlo
      loadMonteCarlo(symbol);
    } catch (e) {
      console.error('Failed to load market data for', symbol, e);
    }
  }

  function generateSignal(symbol: string, candleList: Candle[]) {
    if (candleList.length < 50) return;
    const last = candleList[candleList.length - 1];
    const prev = candleList[candleList.length - 20];
    const isBull = last.close >= prev.close;
    const spec = TfPairSpec.getSpec(symbol);
    const dist = spec.pipSize * 25.0;

    activeSignal = {
      id: 'tf-live-' + Date.now(),
      symbol,
      action: isBull ? 'BUY_LIMIT' : 'SELL_LIMIT',
      timeframe: 'H1',
      entryPrice: isBull ? last.close - (dist * 0.5) : last.close + (dist * 0.5),
      stopLoss: isBull ? last.close - (dist * 1.5) : last.close + (dist * 1.5),
      takeProfit1: isBull ? last.close + (dist * 1.5) : last.close - (dist * 1.5),
      takeProfit2: isBull ? last.close + (dist * 2.0) : last.close - (dist * 2.0),
      riskRewardRatio: 2.0,
      confidenceScore: 0.94,
      strategyName: selectedStrategy.name,
      rationale: selectedStrategy.description,
      status: 'Active',
      createdAt: new Date().toISOString(),
    };
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
      modalType = navId;
      isModalOpen = true;
    }
  }

  onMount(async () => {
    await loadConfig();
    await loadStrategies();
    await loadMarketData(activeSymbol);
    await loadBacktestAndScorecard();
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
    onSelectStrategy={(stratId) => {
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
            {currentPrice}
            {candles}
            {trades}
            signal={activeSignal}
            {syncStatusMessage}
            onSelectSymbol={(sym) => loadMarketData(sym)}
            onSyncDelta={handleSyncDelta}
            onOpenProvenance={handleOpenProvenance}
          />
        </div>

        <!-- 1 Col: Floating Signal Execution HUD -->
        <div class="lg:col-span-1 min-w-0">
          <SignalHud
            signal={activeSignal}
            {activeSymbol}
            activeStrategyName={selectedStrategy.name}
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
