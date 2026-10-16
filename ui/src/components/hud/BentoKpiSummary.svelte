<script lang="ts">
  import { Award, CheckCircle2, ShieldCheck, TrendingUp } from '@lucide/svelte';

  interface Props {
    currentMonthLabel?: string;
    currentMonthVp?: number;
    currentMonthTrades?: number;
    targetPips?: number;
    valuedPips?: number;
    scorecardScore?: number;
    scorecardTier?: string;
    scorecardPillars?: any[];
    wferPct?: number;
    totalBars?: number;
  }

  let {
    currentMonthLabel = 'Monthly Goal',
    currentMonthVp = 0.0,
    currentMonthTrades = 0,
    targetPips = 300.0,
    valuedPips = 3262.5,
    scorecardScore = 20,
    scorecardTier = 'MASTER_PRIORITY',
    scorecardPillars = [],
    wferPct = 94.8,
    totalBars = 198534
  }: Props = $props();

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
</script>

<div class="flex flex-col gap-3 font-mono">
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
