<script lang="ts">
  import { Award, TrendingUp, ShieldCheck, CheckCircle2, Sparkles, Cpu } from '@lucide/svelte';

  interface Props {
    valuedPips: number;
    targetPips: number;
    scorecardScore: number;
    wferPct: number;
    isTfQualified: boolean;
    activeStrategyName?: string;
  }

  let {
    valuedPips = 951.3,
    targetPips = 300.0,
    scorecardScore = 28,
    wferPct = 94.8,
    isTfQualified = true,
    activeStrategyName = 'TF Pola N Structure Engine'
  }: Props = $props();

  let vpProgress = $derived(Math.min(100, Math.max(0, (valuedPips / targetPips) * 100)));
  let rewardCashIdr = $derived((Math.max(0, valuedPips) * 10000).toLocaleString('id-ID'));
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
    <!-- Card 1: Monthly TF Reward Goal -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md min-w-[240px] flex flex-col justify-between">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-[#787b86] flex items-center gap-1.5">
          <Award class="w-3.5 h-3.5 text-[#f5c344]" /> TF Reward (Monthly)
        </span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#f5c344]/10 border border-[#f5c344]/30 text-[#f5c344] font-bold">
          Tier 1-4
        </span>
      </div>

      <div class="flex items-baseline gap-1.5 my-1.5">
        <span class="text-xl font-black font-mono {valuedPips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
          {valuedPips >= 0 ? `+${valuedPips.toFixed(1)}` : valuedPips.toFixed(1)}
        </span>
        <span class="text-xs font-mono text-[#787b86]">/ {targetPips.toFixed(1)} VP</span>
      </div>

      <div class="w-full bg-[#131722] h-1.5 rounded-full overflow-hidden">
        <div class="bg-gradient-to-r from-[#2962ff] to-[#089981] h-full rounded-full transition-all duration-500" style="width: {vpProgress}%;"></div>
      </div>

      <div class="text-[11px] {valuedPips >= targetPips ? 'text-[#089981]' : 'text-[#787b86]'} mt-1.5 font-medium flex items-center gap-1">
        <CheckCircle2 class="w-3.5 h-3.5" /> Target: Rp {rewardCashIdr} ({valuedPips >= targetPips ? 'Ready' : 'In Progress'})
      </div>
    </div>

    <!-- Card 2: 7-Pillar Priority Score -->
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md min-w-[240px] flex flex-col justify-between">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-[#787b86] flex items-center gap-1.5">
          <ShieldCheck class="w-3.5 h-3.5 text-[#089981]" /> 7-Pillar Scorecard
        </span>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#089981]/10 border border-[#089981]/30 text-[#089981] font-bold">
          Legend Priority
        </span>
      </div>

      <div class="flex items-baseline gap-1.5 my-1.5">
        <span class="text-xl font-black font-mono text-[#089981]">{scorecardScore} / 28</span>
        <span class="text-xs font-mono text-[#787b86]">Points (100%)</span>
      </div>

      <div class="flex items-center gap-1 py-0.5">
        {#each ['RF', 'PF', 'PM', 'LR', 'PR', 'LG', 'SB'] as pillar}
          <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 rounded bg-[#089981]/20 text-[#089981] border border-[#089981]/30">
            {pillar}
          </span>
        {/each}
      </div>

      <div class="text-[11px] text-[#787b86] mt-1">
        Max Revenue Share Eligible
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
        <span class="inline-flex items-center gap-1 text-[10px] font-medium text-[#089981] bg-[#131722] px-2 py-0.5 rounded border border-[#089981]/30">
          <span class="w-1.5 h-1.5 rounded-full bg-[#089981] animate-pulse"></span> Sangat Robust di Data Buta
        </span>
      </div>

      <div class="text-[11px] text-[#787b86] mt-1 font-mono">
        103.556 Bar Pasar Nyata
      </div>
    </div>
  </div>
</header>
