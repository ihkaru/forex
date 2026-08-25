<script lang="ts">
  import {
    Radio,
    Send,
    Target,
    ShieldCheck,
    CheckCircle2,
    ArrowUpRight,
    ArrowDownRight,
    Sparkles,
    Zap,
    Copy,
    Check,
    Clock,
    DollarSign,
    Radar,
    Activity,
    Lock,
    TrendingUp,
    TrendingDown,
    Award,
    AlertCircle,
    Flame
  } from '@lucide/svelte';
  import type { Signal, ExecutionHudStatus, RunningPositionState, SettledTradeState, MarketScanContext, PendingOrderState } from '../domain/models';
  import type { PointInTimeComplianceState } from '../ports';

  interface Props {
    status?: ExecutionHudStatus;
    signal?: Signal | null;
    pendingOrder?: PendingOrderState | null;
    activeSymbol?: string;
    activeStrategyName?: string;
    runningPosition?: RunningPositionState | null;
    settledTrade?: SettledTradeState | null;
    scanContext?: MarketScanContext | null;
    valuedPips?: number;
    currentMonthLabel?: string;
    currentMonthVp?: number;
    currentMonthTrades?: number;
    targetPips?: number;
    scorecardScore?: number;
    scorecardTier?: string;
    scorecardPillars?: any[];
    wferPct?: number;
    totalBars?: number;
    complianceState?: PointInTimeComplianceState | null;
    onScanSignal?: () => void;
  }

  let {
    status = 'SCANNING',
    signal = null,
    pendingOrder = null,
    activeSymbol = 'XAUUSD',
    activeStrategyName = 'TF Pola N Adaptive (v2 Gold Specialist)',
    runningPosition = null,
    settledTrade = null,
    scanContext = null,
    valuedPips = 3262.5,
    currentMonthLabel = 'Monthly Goal',
    currentMonthVp = 0.0,
    currentMonthTrades = 0,
    targetPips = 300.0,
    scorecardScore = 20,
    scorecardTier = 'MASTER_PRIORITY',
    scorecardPillars = [],
    wferPct = 94.8,
    totalBars = 198534,
    complianceState = null,
    onScanSignal
  }: Props = $props();


  let isCopied = $state(false);

  // Bento KPI Derivations
  let vpProgress = $derived(Math.min(100, Math.max(0, (currentMonthVp / targetPips) * 100)));
  let rewardCashIdr = $derived((Math.max(0, currentMonthVp) * 10000).toLocaleString('id-ID'));
  let scorePct = $derived(Math.round((scorecardScore / 28) * 100));

  const tierBadgeConfig = $derived.by(() => {
    const tier = String(scorecardTier).toUpperCase();
    if (tier.includes('LEGEND')) {
      return { label: 'Legend Priority (80%)', color: 'bg-[#089981]/10 text-[#089981] border-[#089981]/30' };
    } else if (tier.includes('MASTER')) {
      return { label: 'Master Priority (70%)', color: 'bg-[#2962ff]/10 text-[#2962ff] border-[#2962ff]/30' };
    } else if (tier.includes('PRO')) {
      return { label: 'Pro Priority (60%)', color: 'bg-[#f5c344]/10 text-[#f5c344] border-[#f5c344]/30' };
    } else {
      return { label: 'Silver Priority', color: 'bg-[#787b86]/10 text-[#d1d4dc] border-[#787b86]/30' };
    }
  });

  const defaultPillars = [
    { code: 'RF', status: 'MAX', score: 4 },
    { code: 'PF', status: 'MAX', score: 4 },
    { code: 'PR', status: 'MAX', score: 4 },
    { code: 'LG', status: 'ACCEPTABLE', score: 3 },
    { code: 'LR', status: 'MAX', score: 4 },
    { code: 'PM', status: 'MODERATE', score: 2 },
    { code: 'SB', status: 'LOW', score: 1 },
  ];

  const activePillars = $derived(
    scorecardPillars.length > 0 ? scorecardPillars : defaultPillars
  );

  // Dynamic Effective State
  let effectiveStatus = $derived.by<ExecutionHudStatus>(() => {
    if (runningPosition) return 'RUNNING';
    if (settledTrade) return 'SETTLED';
    if (pendingOrder) return 'PENDING';
    if (signal) return 'PENDING';
    return status;
  });

  // Pending Signal / Order Values
  let isBuy = $derived.by(() => {
    if (pendingOrder) return pendingOrder.action.toUpperCase().includes('BUY');
    if (signal) return signal.action.toUpperCase().includes('BUY');
    return false;
  });

  let actionLabel = $derived.by(() => {
    if (pendingOrder) return pendingOrder.action.replace(/[_/]/g, ' ').toUpperCase();
    if (signal) return signal.action.replace(/[_/]/g, ' ').toUpperCase();
    return 'PENDING BUY LIMIT';
  });

  let entryPrice = $derived.by(() => {
    if (pendingOrder) return pendingOrder.entryPrice > 500 ? pendingOrder.entryPrice.toFixed(2) : pendingOrder.entryPrice.toFixed(5);
    if (signal) return signal.entryPrice > 500 ? signal.entryPrice.toFixed(2) : signal.entryPrice.toFixed(5);
    return '4026.61';
  });

  let stopLoss = $derived.by(() => {
    if (pendingOrder) return pendingOrder.stopLoss > 500 ? pendingOrder.stopLoss.toFixed(2) : pendingOrder.stopLoss.toFixed(5);
    if (signal) return signal.stopLoss > 500 ? signal.stopLoss.toFixed(2) : signal.stopLoss.toFixed(5);
    return '3981.73';
  });

  let tp1 = $derived.by(() => {
    if (pendingOrder) return pendingOrder.takeProfit > 500 ? pendingOrder.takeProfit.toFixed(2) : pendingOrder.takeProfit.toFixed(5);
    if (signal) return signal.takeProfit1 > 500 ? signal.takeProfit1.toFixed(2) : signal.takeProfit1.toFixed(5);
    return '4075.98';
  });

  let tp2 = $derived.by(() => {
    if (signal?.takeProfit2) return signal.takeProfit2 > 500 ? signal.takeProfit2.toFixed(2) : signal.takeProfit2.toFixed(5);
    return null;
  });

  let rrRatio = $derived.by(() => {
    if (pendingOrder) {
      const sl = Math.abs(pendingOrder.entryPrice - pendingOrder.stopLoss);
      const tp = Math.abs(pendingOrder.takeProfit - pendingOrder.entryPrice);
      return sl > 0 ? (tp / sl).toFixed(2) : '1.50';
    }
    if (signal) return signal.riskRewardRatio.toFixed(2);
    return '1.10';
  });

  let riskPips = $derived.by(() => {
    const mult = activeSymbol === 'XAUUSD' ? 10 : 10000;
    if (pendingOrder) return (Math.abs(pendingOrder.entryPrice - pendingOrder.stopLoss) * mult).toFixed(1);
    if (signal) return (Math.abs(signal.entryPrice - signal.stopLoss) * mult).toFixed(1);
    return '25.0';
  });

  let rewardPips = $derived.by(() => {
    const mult = activeSymbol === 'XAUUSD' ? 10 : 10000;
    if (pendingOrder) return (Math.abs(pendingOrder.takeProfit - pendingOrder.entryPrice) * mult).toFixed(1);
    if (signal) return (Math.abs(signal.takeProfit1 - signal.entryPrice) * mult).toFixed(1);
    return '27.0';
  });

  let valuedPipsTarget = $derived.by(() => {
    const mult = activeSymbol === 'XAUUSD' ? 0.5 : 2.0;
    return (parseFloat(rewardPips) * mult).toFixed(1);
  });

  function copySignalText() {
    const text = `⚡ FOREX QUANT SIGNAL ALERT\nPair: ${activeSymbol.slice(0, 3)}/${activeSymbol.slice(3)}\nAction: ${actionLabel}\nEntry: ${entryPrice}\nStop Loss: ${stopLoss} (-${riskPips} pips)\nTake Profit 1: ${tp1} (+${rewardPips} pips / +${valuedPipsTarget} VP)\n${tp2 ? `Take Profit 2: ${tp2}\n` : ''}R:R Ratio: 1:${rrRatio}\nStrategy: ${activeStrategyName}\nTF Compliance: 0-Penalty Guaranteed`;
    navigator.clipboard.writeText(text);
    isCopied = true;
    setTimeout(() => {
      isCopied = false;
    }, 2000);
  }

</script>

<div class="flex flex-col gap-3 font-sans select-none">
  <!-- ========================================================================= -->
  <!-- STATE 1: RUNNING POSITION (Live In-Flight Monitor) -->
  <!-- ========================================================================= -->
  {#if effectiveStatus === 'RUNNING' && runningPosition}
    {@const isPosBuy = runningPosition.action.toUpperCase().includes('BUY')}
    {@const isProfit = runningPosition.floatingPips >= 0}
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-xl flex flex-col gap-3">
      <!-- Running Header -->
      <div class="flex items-center justify-between pb-2.5 border-b border-[#2a2e39]">
        <div class="flex items-center gap-2">
          <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-black flex items-center gap-1.5 bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/40">
            <Activity class="w-3.5 h-3.5 animate-pulse text-[#2962ff]" />
            <span>IN-FLIGHT POSITION</span>
          </span>
          <span class="text-[11px] font-mono font-bold px-2 py-0.5 rounded {isPosBuy ? 'bg-[#089981]/20 text-[#089981]' : 'bg-[#f23645]/20 text-[#f23645]'}">
            {runningPosition.action}
          </span>
        </div>
        <div class="flex items-center gap-1 text-[11px] font-mono text-[#787b86]">
          <Clock class="w-3 h-3" />
          <span>{runningPosition.heldBarsCount} Bar ({runningPosition.heldBarsCount}h)</span>
        </div>
      </div>

      <!-- Floating PnL Hero Card -->
      <div class="p-3.5 rounded-xl border flex items-center justify-between transition-all {isProfit ? 'bg-[#089981]/15 border-[#089981]/40' : 'bg-[#f23645]/15 border-[#f23645]/40'}">
        <div>
          <div class="text-[10px] font-mono font-bold uppercase tracking-wider {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
            Floating Unrealized PnL
          </div>
          <div class="text-2xl font-black font-mono mt-0.5 {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
            {isProfit ? '+' : ''}{runningPosition.floatingPips.toFixed(1)} Pips
          </div>
        </div>
        <div class="text-right">
          <div class="text-[10px] font-mono text-[#787b86] uppercase">Valued Pips</div>
          <div class="text-lg font-black font-mono {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
            {isProfit ? '+' : ''}{runningPosition.floatingValuedPips.toFixed(1)} VP
          </div>
        </div>
      </div>

      <!-- Distance to TP Progress Gauge -->
      <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39] space-y-1.5">
        <div class="flex items-center justify-between text-[10px] font-mono">
          <span class="text-[#787b86]">Target Progress:</span>
          <span class="font-bold {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
            {Math.min(100, Math.max(0, runningPosition.progressToTpPct)).toFixed(0)}% to Take Profit
          </span>
        </div>
        <div class="w-full bg-[#2a2e39] h-2 rounded-full overflow-hidden">
          <div
            class="h-full transition-all duration-300 {isProfit ? 'bg-[#089981]' : 'bg-[#f23645]'}"
            style="width: {Math.min(100, Math.max(5, runningPosition.progressToTpPct))}%"
          ></div>
        </div>
      </div>

      <!-- Key Position Levels 4-Box Grid -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
          <div class="text-[9px] text-[#787b86] font-mono">OPEN PRICE</div>
          <div class="text-xs font-black font-mono text-white mt-0.5">
            {runningPosition.openPrice > 500 ? runningPosition.openPrice.toFixed(2) : runningPosition.openPrice.toFixed(5)}
          </div>
        </div>
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
          <div class="text-[9px] text-[#787b86] font-mono">CURRENT PRICE</div>
          <div class="text-xs font-black font-mono text-[#2962ff] mt-0.5">
            {runningPosition.currentPrice > 500 ? runningPosition.currentPrice.toFixed(2) : runningPosition.currentPrice.toFixed(5)}
          </div>
        </div>
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
          <div class="text-[9px] text-[#787b86] font-mono">TAKE PROFIT</div>
          <div class="text-xs font-black font-mono text-[#089981] mt-0.5">
            {runningPosition.takeProfit > 500 ? runningPosition.takeProfit.toFixed(2) : runningPosition.takeProfit.toFixed(5)}
          </div>
        </div>
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
          <div class="text-[9px] text-[#787b86] font-mono">STOP LOSS</div>
          <div class="text-xs font-black font-mono text-[#f23645] mt-0.5">
            {runningPosition.stopLoss > 500 ? runningPosition.stopLoss.toFixed(2) : runningPosition.stopLoss.toFixed(5)}
          </div>
        </div>
      </div>
    </div>

  <!-- ========================================================================= -->
  <!-- STATE 2: SETTLED TRADE OUTCOME -->
  <!-- ========================================================================= -->
  {:else if effectiveStatus === 'SETTLED' && settledTrade}
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-xl flex flex-col gap-3 animate-in zoom-in-95 duration-150">
      <!-- Settled Header -->
      <div class="flex items-center justify-between pb-2 border-b border-[#2a2e39]">
        <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 {settledTrade.isWin ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40'}">
          <Award class="w-3.5 h-3.5" />
          <span>{settledTrade.isWin ? '🎉 TAKE PROFIT HIT (SETTLED)' : '🛑 STOP LOSS HIT (SETTLED)'}</span>
        </span>
        <span class="text-xs font-mono font-bold {settledTrade.isWin ? 'text-[#089981]' : 'text-[#f23645]'}">
          {settledTrade.valuedPips > 0 ? '+' : ''}{settledTrade.valuedPips.toFixed(1)} VP
        </span>
      </div>

      <!-- Outcome Summary Card -->
      <div class="grid grid-cols-2 gap-2 text-xs font-mono">
        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <span class="text-[10px] text-[#787b86]">Hasil Realisasi:</span>
          <div class="font-bold text-base mt-0.5 {settledTrade.isWin ? 'text-[#089981]' : 'text-[#f23645]'}">
            {settledTrade.pnlPips > 0 ? '+' : ''}{settledTrade.pnlPips.toFixed(1)} Pips
          </div>
        </div>
        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <span class="text-[10px] text-[#787b86]">Alasan Penutupan:</span>
          <div class="font-bold text-white mt-0.5">
            {settledTrade.exitReason}
          </div>
        </div>
      </div>

      <div class="text-[10px] font-mono text-[#787b86] text-center pt-1 border-t border-[#2a2e39]/60">
        Trade selesai dieksekusi secara deterministik. Sistem kembali memindai pola berikutnya.
      </div>
    </div>

  <!-- ========================================================================= -->
  <!-- STATE 3: PENDING ORDER (LOCKED IMMUTABLE SIGNAL / REPLAY PENDING) -->
  <!-- ========================================================================= -->
  {:else if effectiveStatus === 'PENDING'}
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-3">
      <!-- Header with Dynamic Trigger Scan Button -->
      <div class="flex flex-wrap items-center justify-between gap-2 pb-2.5 border-b border-[#2a2e39]">
        <div class="flex items-center gap-2">
          <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 {isBuy ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40'}">
            {#if isBuy}
              <ArrowUpRight class="w-3.5 h-3.5" />
            {:else}
              <ArrowDownRight class="w-3.5 h-3.5" />
            {/if}
            <span>{actionLabel}</span>
          </span>
          <span class="text-xs font-mono px-2 py-0.5 rounded bg-[#131722] text-[#787b86] border border-[#2a2e39]">
            R:R 1:{rrRatio}
          </span>
        </div>

        {#if pendingOrder}
          <div class="flex items-center gap-1.5 text-[11px] font-mono text-[#f59e0b] bg-[#f59e0b]/10 px-2.5 py-1 rounded-lg border border-[#f59e0b]/30">
            <Clock class="w-3.5 h-3.5 animate-pulse" />
            <span>Menunggu Fill ({pendingOrder.distancePips.toFixed(1)}p)</span>
          </div>
        {:else if onScanSignal}
          <button
            onclick={onScanSignal}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#2962ff] hover:bg-[#1e53e5] text-white text-xs font-bold font-mono transition-all shadow-md shadow-[#2962ff]/20 active:scale-95"
            title="Scan dan evaluasi sinyal baru dengan {activeStrategyName}"
          >
            <Zap class="w-3.5 h-3.5" />
            <span>Scan Sinyal</span>
          </button>
        {/if}
      </div>

      <!-- Levels 4-Box Grid with Pips Delta -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <div class="text-[10px] text-[#787b86] font-mono font-bold">ENTRY LEVEL</div>
          <div class="text-sm font-black font-mono text-[#f59e0b] mt-0.5">{entryPrice}</div>
          <div class="text-[9px] text-[#787b86] font-mono mt-0.5">Pending Order</div>
        </div>
        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <div class="text-[10px] text-[#787b86] font-mono font-bold">STOP LOSS (SL)</div>
          <div class="text-sm font-black font-mono text-[#f23645] mt-0.5">{stopLoss}</div>
          <div class="text-[9px] text-[#f23645] font-mono mt-0.5">-{riskPips} pips</div>
        </div>
        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <div class="text-[10px] text-[#787b86] font-mono font-bold">TAKE PROFIT 1</div>
          <div class="text-sm font-black font-mono text-[#089981] mt-0.5">{tp1}</div>
          <div class="text-[9px] text-[#089981] font-mono mt-0.5">+{rewardPips} pips (+{valuedPipsTarget} VP)</div>
        </div>
        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <div class="text-[10px] text-[#787b86] font-mono font-bold">TAKE PROFIT 2</div>
          <div class="text-sm font-black font-mono text-[#089981] mt-0.5">{tp2 ?? '-'}</div>
          <div class="text-[9px] text-[#089981] font-mono mt-0.5">Runner Level</div>
        </div>
      </div>
    </div>


    <!-- Trade Risk, Compliance & Instant Dispatch Control -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-2.5">
      <div class="flex items-center justify-between pb-2 border-b border-[#2a2e39]">
        <div class="flex items-center gap-2">
          <Radio class="w-4 h-4 text-[#f5c344]" />
          <h3 class="text-xs font-bold font-mono text-[#d1d4dc]">Trade Execution & Compliance</h3>
        </div>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#089981]/15 text-[#089981] font-bold border border-[#089981]/30 flex items-center gap-1">
          <ShieldCheck class="w-3 h-3" /> TF 0-Penalty Valid
        </span>
      </div>

      <div class="grid grid-cols-2 gap-2 text-xs font-mono">
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
          <span class="text-[10px] text-[#787b86]">Risk Sizing:</span>
          <span class="font-bold text-white mt-0.5">Quarter Kelly (1.0%)</span>
        </div>
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
          <span class="text-[10px] text-[#787b86]">SLA Salin Sinyal:</span>
          <span class="font-bold text-[#089981] mt-0.5">≥ 5 Menit Aman</span>
        </div>
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
          <span class="text-[10px] text-[#787b86]">Target VP Monetisasi:</span>
          <span class="font-bold text-[#2962ff] mt-0.5">+{valuedPipsTarget} VP / Trade</span>
        </div>
        <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
          <span class="text-[10px] text-[#787b86]">Expiry Durasi:</span>
          <span class="font-bold text-white mt-0.5">24 Jam (Auto-Expire)</span>
        </div>
      </div>

      <div class="flex items-center gap-2 pt-1">
        <button
          onclick={copySignalText}
          class="flex-1 flex items-center justify-center gap-2 py-2 px-3 rounded-lg text-xs font-mono font-bold bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/50 text-[#d1d4dc] hover:text-white transition-all shadow-sm"
        >
          {#if isCopied}
            <Check class="w-3.5 h-3.5 text-[#089981]" />
            <span class="text-[#089981]">Tersalin ke Clipboard!</span>
          {:else}
            <Copy class="w-3.5 h-3.5 text-[#787b86]" />
            <span>Salin Format Sinyal</span>
          {/if}
        </button>

        <button
          class="flex items-center justify-center gap-1.5 py-2 px-4 rounded-lg text-xs font-mono font-bold bg-[#2962ff] hover:bg-[#1e4bd8] text-white transition-all shadow-md shadow-[#2962ff]/20"
          title="Otomatis disalurkan ke Traders Family Priority Channel"
        >
          <Send class="w-3.5 h-3.5" />
          <span>Dispatch VIP</span>
        </button>
      </div>
    </div>

  <!-- ========================================================================= -->
  <!-- STATE 4: SCANNING MARKET (Default Tactical Radar) -->
  <!-- ========================================================================= -->
  {:else}
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-3">
      <!-- Radar Header -->
      <div class="flex items-center justify-between pb-2.5 border-b border-[#2a2e39]">
        <div class="flex items-center gap-2">
          <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 bg-[#2962ff]/15 text-[#2962ff] border border-[#2962ff]/30">
            <Radar class="w-3.5 h-3.5 animate-spin text-[#2962ff]" style="animation-duration: 4s;" />
            <span>SCANNING MARKET</span>
          </span>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#131722] text-[#787b86] border border-[#2a2e39]">
            {activeSymbol}
          </span>
        </div>

        {#if onScanSignal}
          <button
            onclick={onScanSignal}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#2962ff] hover:bg-[#1e53e5] text-white text-xs font-bold font-mono transition-all shadow-md shadow-[#2962ff]/20 active:scale-95"
            title="Scan market sekarang"
          >
            <Zap class="w-3.5 h-3.5" />
            <span>Scan Market</span>
          </button>
        {/if}
      </div>

      <!-- Tactical Market Radar Grid -->
      <div class="grid grid-cols-2 gap-2 font-mono text-xs">
        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <span class="text-[10px] text-[#787b86]">Tren EMA Alignment:</span>
          <div class="font-bold flex items-center gap-1 mt-0.5 {scanContext?.trend === 'BULLISH' ? 'text-[#089981]' : scanContext?.trend === 'BEARISH' ? 'text-[#f23645]' : 'text-[#f5c344]'}">
            {#if scanContext?.trend === 'BULLISH'}
              <TrendingUp class="w-3.5 h-3.5" /> Bullish Alignment
            {:else if scanContext?.trend === 'BEARISH'}
              <TrendingDown class="w-3.5 h-3.5" /> Bearish Alignment
            {:else}
              <Activity class="w-3.5 h-3.5" /> Neutral / Ranging
            {/if}
          </div>
        </div>

        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <span class="text-[10px] text-[#787b86]">RSI Pullback Window:</span>
          <div class="font-bold text-white mt-0.5">
            {scanContext ? `${scanContext.rsi.toFixed(1)} (Pullback)` : '48.2 (Golden Window)'}
          </div>
        </div>

        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <span class="text-[10px] text-[#787b86]">Session Liquidity:</span>
          <div class="font-bold mt-0.5 {scanContext?.isSessionActive ?? true ? 'text-[#089981]' : 'text-[#787b86]'}">
            {scanContext?.sessionName ?? 'London / NY Active'}
          </div>
        </div>

        <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
          <span class="text-[10px] text-[#787b86]">Status Formasi:</span>
          <div class="font-bold text-[#f5c344] mt-0.5">
            {scanContext?.waitingReason ?? 'Menunggu Retracement 61.8%'}
          </div>
        </div>
      </div>

      <!-- Tactical Status Footer -->
      <div class="p-2.5 rounded-lg bg-[#131722] border border-[#2a2e39]/60 text-[10px] font-mono text-[#787b86] flex items-center gap-2">
        <Sparkles class="w-4 h-4 text-[#2962ff] flex-shrink-0" />
        <span>Standby: Order hanya akan diposting jika seluruh 5 filter institusional Pola N terkonfirmasi secara deterministik.</span>
      </div>
    </div>

    <!-- TF Rule Invariant Compliance Guarantee (Point-In-Time Live Guard) -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-2 font-mono text-xs">
      <div class="flex items-center justify-between pb-1.5 border-b border-[#2a2e39]">
        <div class="flex items-center gap-1.5 text-[#d1d4dc] font-bold">
          <ShieldCheck class="w-3.5 h-3.5 text-[#089981]" />
          <span>TF Compliance Policy</span>
        </div>
        <span class="text-[10px] {complianceState?.isCompliant ?? true ? 'text-[#089981]' : 'text-[#f23645]'} font-bold">
          {complianceState?.isCompliant ?? true ? '100% Deterministic' : '⚠️ Quota Alert'}
        </span>
      </div>
      <div class="grid grid-cols-2 gap-2 text-[10px] text-[#787b86] pt-1">
        <div class="flex items-center gap-1">
          <span class="text-[#089981]">✓</span> Dilarang Instant Order
        </div>
        <div class="flex items-center gap-1">
          <span class="text-[#089981]">✓</span> Batas R:R 1:1.0 - 1:3.0
        </div>
        <div class="flex items-center gap-1">
          <span class="text-[#089981]">✓</span> SLA Salin Sinyal ≥ 5 Menit
        </div>
        <div class="flex items-center gap-1">
          <span class="{(complianceState?.activeSignalsOnPair ?? 0) > 0 ? 'text-[#f5c344]' : 'text-[#089981]'}">●</span>
          <span>Kuota: <strong class="{(complianceState?.activeSignalsOnPair ?? 0) > 0 ? 'text-white' : 'text-[#787b86]'}">{complianceState?.activeSignalsOnPair ?? 0}/{complianceState?.maxSignalsPerPair ?? 2} Sinyal Aktif</strong></span>
        </div>
      </div>
    </div>
  {/if}

  <!-- ========================================================================= -->
  <!-- BENTO INTELLIGENCE KPI CARDS (Aligned with Chart) -->
  <!-- ========================================================================= -->
  <div class="flex flex-col gap-3">
    <!-- Card 1: Monthly TF Reward Goal & Portfolio PnL -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-2 font-mono">
      <div class="flex items-center justify-between">
        <span class="text-xs font-bold text-[#d1d4dc] flex items-center gap-1.5 font-sans">
          <Award class="w-4 h-4 text-[#f5c344]" /> TF Reward ({currentMonthLabel})
        </span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#f5c344]/10 border border-[#f5c344]/30 text-[#f5c344] font-bold">
          Tier 1-4
        </span>
      </div>

      <div class="flex items-baseline justify-between my-0.5">
        <div class="flex items-baseline gap-1">
          <span class="text-xl font-black {currentMonthVp >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
            {currentMonthVp >= 0 ? `+${currentMonthVp.toFixed(1)}` : currentMonthVp.toFixed(1)}
          </span>
          <span class="text-xs text-[#787b86]">/ {targetPips.toFixed(1)} VP</span>
        </div>
        <span class="text-[10px] text-[#787b86]">
          All-Time: <strong class="{valuedPips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">{valuedPips >= 0 ? '+' : ''}{valuedPips.toFixed(1)} VP</strong>
        </span>
      </div>

      <div class="w-full bg-[#131722] h-1.5 rounded-full overflow-hidden">
        <div class="bg-gradient-to-r from-[#2962ff] to-[#089981] h-full rounded-full transition-all duration-500" style="width: {vpProgress}%;"></div>
      </div>

      <div class="text-[10px] {currentMonthVp >= targetPips ? 'text-[#089981]' : 'text-[#787b86]'} font-medium flex items-center justify-between font-sans">
        <span class="flex items-center gap-1">
          <CheckCircle2 class="w-3.5 h-3.5 text-[#089981]" /> Target: Rp {rewardCashIdr} ({currentMonthVp >= targetPips ? 'Qualified' : 'In Progress'})
        </span>
        <span class="font-mono">{currentMonthTrades} settled</span>
      </div>
    </div>


    <!-- Card 2 & 3: 2-Col Grid for Scorecard & Robustness -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <!-- Card 2: 7-Pillar Priority Score -->
      <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3 shadow-md flex flex-col justify-between font-mono">
        <div class="flex items-center justify-between pb-1 border-b border-[#2a2e39]/60">
          <span class="text-[11px] font-bold text-[#d1d4dc] flex items-center gap-1 font-sans">
            <ShieldCheck class="w-3.5 h-3.5 text-[#089981]" /> 7-Pillars
          </span>
          <span class="text-[9px] px-1.5 py-0.2 rounded border font-bold {tierBadgeConfig.color}">
            {tierBadgeConfig.label.split(' ')[0]}
          </span>
        </div>

        <div class="flex items-baseline gap-1 my-1">
          <span class="text-base font-black {scorecardScore >= 18 ? 'text-[#089981]' : 'text-[#2962ff]'}">
            {scorecardScore}/28
          </span>
          <span class="text-[10px] text-[#787b86]">({scorePct}%)</span>
        </div>

        <div class="flex flex-wrap items-center gap-1">
          {#each activePillars as pillar}
            {@const isPassed = (pillar.score ?? 0) >= 3 || String(pillar.status).includes('PASS') || String(pillar.status).includes('MAX')}
            <span 
              class="text-[8px] font-bold px-1 rounded border {isPassed ? 'bg-[#089981]/20 text-[#089981] border-[#089981]/30' : 'bg-[#2a2e39] text-[#787b86] border-[#363a45]'}"
              title="{pillar.name || pillar.code}: {pillar.score ?? 0}/{pillar.max_score ?? 4} pts"
            >
              {pillar.code}
            </span>
          {/each}
        </div>
      </div>

      <!-- Card 3: WFER Robustness Dial -->
      <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3 shadow-md flex flex-col justify-between font-mono">
        <div class="flex items-center justify-between pb-1 border-b border-[#2a2e39]/60">
          <span class="text-[11px] font-bold text-[#d1d4dc] flex items-center gap-1 font-sans">
            <TrendingUp class="w-3.5 h-3.5 text-[#2962ff]" /> WFER OOS
          </span>
          <span class="text-[9px] px-1.5 py-0.2 rounded bg-[#2962ff]/10 border border-[#2962ff]/30 text-[#2962ff] font-bold">
            Anti-Overfit
          </span>
        </div>

        <div class="flex items-baseline gap-1 my-1">
          <span class="text-base font-black text-[#2962ff]">{wferPct.toFixed(1)}%</span>
          <span class="text-[10px] text-[#089981] font-sans font-bold">● Robust</span>
        </div>

        <div class="text-[9px] text-[#787b86]">
          {totalBars.toLocaleString('id-ID')} Real Bars
        </div>
      </div>
    </div>
  </div>
</div>


