<script lang="ts">
  import { Award, TrendingUp, ShieldCheck, CheckCircle2, Cpu } from '@lucide/svelte';

  interface Props {
    valuedPips: number;
    currentMonthVp?: number;
    currentMonthTrades?: number;
    targetPips?: number;
    scorecardScore?: number;
    scorecardTier?: string;
    scorecardPillars?: any[];
    wferPct?: number;
    totalBars?: number;
    isTfQualified?: boolean;
    activeStrategyName?: string;
  }

  let {
    valuedPips = -12874.4,
    currentMonthVp = 0.0,
    currentMonthTrades = 0,
    targetPips = 300.0,
    scorecardScore = 12,
    scorecardTier = 'SILVER_PRIORITY',
    scorecardPillars = [],
    wferPct = 94.8,
    totalBars = 198534,
    isTfQualified = false,
    activeStrategyName = 'TF Pola N Structure Engine'
  }: Props = $props();

  let vpProgress = $derived(Math.min(100, Math.max(0, (currentMonthVp / targetPips) * 100)));
  let rewardCashIdr = $derived((Math.max(0, currentMonthVp) * 10000).toLocaleString('id-ID'));
  let scorePct = $derived(Math.round((scorecardScore / 28) * 100));

  const tierBadgeConfig = $derived.by(() => {
    const tier = String(scorecardTier).toUpperCase();
    if (tier.includes('LEGEND')) {
      return { label: 'Legend Priority', color: 'bg-[#089981]/10 text-[#089981] border-[#089981]/30' };
    } else if (tier.includes('MASTER')) {
      return { label: 'Master Priority', color: 'bg-[#2962ff]/10 text-[#2962ff] border-[#2962ff]/30' };
    } else if (tier.includes('PRO')) {
      return { label: 'Pro Priority', color: 'bg-[#f5c344]/10 text-[#f5c344] border-[#f5c344]/30' };
    } else {
      return { label: 'Silver Priority', color: 'bg-[#787b86]/10 text-[#d1d4dc] border-[#787b86]/30' };
    }
  });

  const defaultPillars = [
    { code: 'RF', status: 'LOW' },
    { code: 'PF', status: 'LOW' },
    { code: 'PR', status: 'PASSED' },
    { code: 'LG', status: 'LOW' },
    { code: 'LR', status: 'PASSED' },
    { code: 'PM', status: 'LOW' },
    { code: 'SB', status: 'PASSED' },
  ];

  const activePillars = $derived(
    scorecardPillars.length > 0 ? scorecardPillars : defaultPillars
  );
</script>

<header class="flex flex-col xl:flex-row items-start xl:items-center justify-between gap-4 pb-4 border-b border-[#2a2e39] font-sans">
  <!-- Title & Core Strategy -->
  <div>
    <div class="flex items-center gap-2.5">
      <div class="p-1.5 rounded-lg bg-[#2962ff]/10 border border-[#2962ff]/30 text-[#2962ff]">
        <Cpu class="w-5 h-5" />
      </div>
      <h1 class="text-xl font-bold tracking-tight text-[#d1d4dc] font-mono">Forex Quantitative Signal Terminal</h1>
    </div>
    <p class="text-xs text-[#787b86] mt-1 flex items-center gap-2">
      <span class="text-[#2962ff] font-semibold">{activeStrategyName}</span>
      <span class="text-[#363a45]">•</span>
      <span>TradingView Dark Engine</span>
      <span class="text-[#363a45]">•</span>
      <span>4-Tier Valued Pips</span>
      <span class="text-[#363a45]">•</span>
      <span class="text-[#089981] font-mono flex items-center gap-1">
        <ShieldCheck class="w-3.5 h-3.5" /> 0-Penalty Active
      </span>
    </p>
  </div>

  <!-- Bento KPI Grid (TradingView Colors) -->
  <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 w-full xl:w-auto">
    <!-- Card 1: Monthly TF Reward Goal & Portfolio PnL -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md min-w-[240px] flex flex-col justify-between">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-[#787b86] flex items-center gap-1.5">
          <Award class="w-3.5 h-3.5 text-[#f5c344]" /> TF Reward (Monthly)
        </span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#f5c344]/10 border border-[#f5c344]/30 text-[#f5c344] font-bold">
          Tier 1-4
        </span>
      </div>

      <div class="flex items-baseline justify-between my-1.5">
        <div class="flex items-baseline gap-1">
          <span class="text-xl font-black font-mono {currentMonthVp >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
            {currentMonthVp >= 0 ? `+${currentMonthVp.toFixed(1)}` : currentMonthVp.toFixed(1)}
          </span>
          <span class="text-xs font-mono text-[#787b86]">/ {targetPips.toFixed(1)} VP</span>
        </div>
        <span class="text-[10px] font-mono text-[#787b86] text-right" title="10-Year Cumulative All-Time Portfolio VP">
          All-Time: <strong class="{valuedPips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">{valuedPips >= 0 ? '+' : ''}{valuedPips.toFixed(1)} VP</strong>
        </span>
      </div>

      <div class="w-full bg-[#131722] h-1.5 rounded-full overflow-hidden">
        <div class="bg-gradient-to-r from-[#2962ff] to-[#089981] h-full rounded-full transition-all duration-500" style="width: {vpProgress}%;"></div>
      </div>

      <div class="text-[11px] {currentMonthVp >= targetPips ? 'text-[#089981]' : 'text-[#787b86]'} mt-1.5 font-medium flex items-center justify-between">
        <span class="flex items-center gap-1">
          <CheckCircle2 class="w-3.5 h-3.5" /> Target: Rp {rewardCashIdr} ({currentMonthVp >= targetPips ? 'Qualified' : 'In Progress'})
        </span>
        <span class="text-[10px] font-mono text-[#787b86]">{currentMonthTrades} settled</span>
      </div>
    </div>

    <!-- Card 2: 7-Pillar Priority Score -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md min-w-[240px] flex flex-col justify-between">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-[#787b86] flex items-center gap-1.5">
          <ShieldCheck class="w-3.5 h-3.5 text-[#089981]" /> 7-Pillar Scorecard
        </span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded border font-bold {tierBadgeConfig.color}">
          {tierBadgeConfig.label}
        </span>
      </div>

      <div class="flex items-baseline gap-1.5 my-1.5">
        <span class="text-xl font-black font-mono {scorecardScore >= 18 ? 'text-[#089981]' : (scorecardScore >= 12 ? 'text-[#2962ff]' : 'text-[#f5c344]')}">
          {scorecardScore} / 28
        </span>
        <span class="text-xs font-mono text-[#787b86]">Points ({scorePct}%)</span>
      </div>

      <div class="flex items-center gap-1 py-0.5">
        {#each activePillars as pillar}
          {@const isPassed = (pillar.score ?? 0) >= 3 || String(pillar.status).includes('PASS') || String(pillar.status).includes('MAX')}
          <span 
            class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded border transition-colors {isPassed ? 'bg-[#089981]/20 text-[#089981] border-[#089981]/30' : 'bg-[#2a2e39] text-[#787b86] border-[#363a45]'}"
            title="{pillar.name || pillar.code}: {pillar.score ?? 0}/{pillar.max_score ?? 4} pts ({pillar.value_label || pillar.our_value || ''})"
          >
            {pillar.code}
          </span>
        {/each}
      </div>

      <div class="text-[11px] text-[#787b86] mt-1 font-mono">
        {scorecardScore >= 24 ? 'Max 80% Rev-Share Eligible' : (scorecardScore >= 18 ? '70% Rev-Share Eligible' : (scorecardScore >= 12 ? '60% Rev-Share Eligible' : '50% Base Tier'))}
      </div>
    </div>

    <!-- Card 3: WFER Robustness Dial -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md min-w-[240px] flex flex-col justify-between">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-[#787b86] flex items-center gap-1.5">
          <TrendingUp class="w-3.5 h-3.5 text-[#2962ff]" /> Walk-Forward WFER
        </span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#2962ff]/10 border border-[#2962ff]/30 text-[#2962ff] font-bold">
          Anti-Overfit
        </span>
      </div>

      <div class="flex items-baseline gap-1.5 my-1.5">
        <span class="text-xl font-black font-mono text-[#2962ff]">{wferPct.toFixed(1)}%</span>
        <span class="text-xs font-mono text-[#787b86]">OOS Stability</span>
      </div>

      <div class="flex items-center gap-1.5">
        <span class="inline-flex items-center gap-1 text-[10px] font-medium {wferPct >= 70 ? 'text-[#089981] border-[#089981]/30' : 'text-[#f5c344] border-[#f5c344]/30'} bg-[#131722] px-2 py-0.5 rounded border">
          <span class="w-1.5 h-1.5 rounded-full {wferPct >= 70 ? 'bg-[#089981]' : 'bg-[#f5c344]'} animate-pulse"></span>
          {wferPct >= 70 ? 'Sangat Robust di Data Buta' : 'Moderate OOS Ratio'}
        </span>
      </div>

      <div class="text-[11px] text-[#787b86] mt-1 font-mono">
        {totalBars.toLocaleString('id-ID')} Bar Pasar Nyata
      </div>
    </div>
  </div>
</header>
