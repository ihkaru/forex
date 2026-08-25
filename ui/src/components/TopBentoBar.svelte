<script lang="ts">
  import {
    Award,
    TrendingUp,
    ShieldCheck,
    CheckCircle2,
    Cpu,
    Zap,
    ChevronDown,
    Check,
    Layers,
    Search,
    Dices
  } from '@lucide/svelte';
  import type { StrategyDescriptor } from '../ports';

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
    strategies?: StrategyDescriptor[];
    selectedStrategyId?: string;
    onSelectStrategy?: (strategyId: string) => void;
    onOpenModelHub?: () => void;
    onOpenMonteCarlo?: () => void;
  }

  let {
    valuedPips = 3149.6,
    currentMonthVp = 0.0,
    currentMonthTrades = 0,
    targetPips = 300.0,
    scorecardScore = 19,
    scorecardTier = 'MASTER_PRIORITY',
    scorecardPillars = [],
    wferPct = 98.2,
    totalBars = 198534,
    isTfQualified = true,
    strategies = [],
    selectedStrategyId = 'pola-n-v2',
    onSelectStrategy,
    onOpenModelHub,
    onOpenMonteCarlo
  }: Props = $props();

  let isDropdownOpen = $state(false);

  let selectedStrategy = $derived(
    strategies.find((s) => s.id === selectedStrategyId) || strategies[0]
  );

  let isGoldSpecialist = $derived(
    selectedStrategy?.isSpecialist || selectedStrategy?.id === 'pola-n-v2'
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

  function handleSelect(id: string) {
    if (onSelectStrategy) {
      onSelectStrategy(id);
    }
    isDropdownOpen = false;
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') isDropdownOpen = false; }} />

<header class="flex flex-col md:flex-row items-start md:items-center justify-between gap-3 pb-2.5 border-b border-[#2a2e39] font-sans">

  <!-- Left: Title, Strategy Engine Selector & Active Statuses -->
  <div class="flex flex-wrap items-center gap-3">
    <div class="flex items-center gap-2">
      <div class="p-1.5 rounded-lg bg-gradient-to-br from-[#2962ff] to-[#1e4bd8] text-white shadow-md">
        <Cpu class="w-4 h-4" />
      </div>
      <h1 class="text-base font-bold tracking-tight text-white font-mono">
        Forex Quantitative Terminal
      </h1>
    </div>

    <!-- Integrated Strategy Engine Picker Dropdown (TradingView Style) -->
    {#if strategies.length > 0}
      <div class="relative">
        <button
          onclick={() => (isDropdownOpen = !isDropdownOpen)}
          class="flex items-center gap-2 px-2.5 py-1 rounded-lg text-xs font-mono font-bold bg-[#1e222d] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-white transition-all shadow-sm group"
        >
          <span class="px-1.5 py-0.5 rounded bg-[#2962ff]/20 text-[#2962ff] font-extrabold text-[10px] flex items-center gap-1">
            <Zap class="w-3 h-3 text-[#2962ff]" /> fx
          </span>

          <span class="font-bold text-[#d1d4dc] group-hover:text-white transition-colors">
            {selectedStrategy?.name ?? 'Pilih Strategi...'}
          </span>

          {#if isGoldSpecialist}
            <span class="text-[9px] px-1.5 py-0.5 rounded font-extrabold bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40 font-mono">
              ⭐ GOLD SPECIALIST (PF 2.19)
            </span>
          {:else if selectedStrategy}
            <span class="text-[9px] px-1.5 py-0.5 rounded font-bold bg-[#089981]/20 text-[#089981] font-mono">
              {selectedStrategy.winRatePct.toFixed(1)}% WR
            </span>
          {/if}

          <ChevronDown class="w-3.5 h-3.5 text-[#787b86] group-hover:text-white transition-transform {isDropdownOpen ? 'rotate-180 text-[#2962ff]' : ''}" />
        </button>

        <!-- Dropdown Menu Popover -->
        {#if isDropdownOpen}
          <button
            type="button"
            class="fixed inset-0 z-30 bg-transparent border-0 cursor-default"
            onclick={() => (isDropdownOpen = false)}
            aria-label="Close Strategy Dropdown"
          ></button>

          <div class="absolute left-0 mt-2 w-80 sm:w-96 rounded-xl bg-[#1e222d] border border-[#2a2e39] shadow-2xl p-2 z-40 animate-in fade-in zoom-in-95 duration-150">
            <div class="flex items-center justify-between px-2 py-1.5 mb-1 border-b border-[#2a2e39]/60 text-[11px] font-mono text-[#787b86]">
              <span class="font-bold uppercase tracking-wider">TradingView Strategy Models</span>
              <span class="text-[#2962ff] font-semibold">{strategies.length} Ready</span>
            </div>

            <div class="flex flex-col gap-1 max-h-72 overflow-y-auto pr-1">
              {#each strategies as s}
                {@const isCurrent = s.id === selectedStrategyId}
                {@const isGold = s.isSpecialist || s.id === 'pola-n-v2'}
                <button
                  onclick={() => handleSelect(s.id)}
                  class="w-full flex items-start gap-2.5 p-2 rounded-lg text-left transition-all {isCurrent ? 'bg-[#2962ff]/15 border border-[#2962ff]/40 shadow-sm' : 'hover:bg-[#2a2e39] border border-transparent'}"
                >
                  <div class="p-1 rounded bg-[#131722] text-[#2962ff] mt-0.5">
                    <Cpu class="w-3.5 h-3.5 {isCurrent ? 'text-[#2962ff]' : 'text-[#787b86]'}" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between gap-1">
                      <span class="text-xs font-bold font-mono text-white truncate">{s.name}</span>
                      {#if isCurrent}
                        <Check class="w-3.5 h-3.5 text-[#2962ff] shrink-0" />
                      {/if}
                    </div>

                    <div class="text-[10px] text-[#787b86] line-clamp-1 mt-0.5">{s.description}</div>

                    <div class="flex items-center gap-2 mt-1 font-mono text-[9px]">
                      {#if isGold}
                        <span class="px-1.5 py-0.2 rounded font-bold bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40">
                          ⭐ GOLD SPECIALIST
                        </span>
                      {/if}
                      <span class="px-1 rounded bg-[#131722] text-[#d1d4dc]">PF: <strong class="text-[#089981]">{s.profitFactor.toFixed(2)}</strong></span>
                      <span class="px-1 rounded bg-[#131722] text-[#d1d4dc]">WR: <strong class="text-[#2962ff]">{s.winRatePct.toFixed(1)}%</strong></span>
                    </div>
                  </div>
                </button>
              {/each}
            </div>

            {#if onOpenModelHub}
              <div class="pt-2 mt-1 border-t border-[#2a2e39] flex items-center justify-between">
                <button
                  onclick={() => { isDropdownOpen = false; onOpenModelHub(); }}
                  class="w-full py-1.5 px-3 rounded-lg text-xs font-mono font-bold bg-[#2962ff] hover:bg-[#1e4bd8] text-white transition-colors flex items-center justify-center gap-2"
                >
                  <Search class="w-3.5 h-3.5" /> Buka Model Hub Lengkap (Compare Matrix)
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Live Compliance Badges -->
    <div class="hidden sm:flex items-center gap-2 text-xs font-mono text-[#787b86]">
      <span class="text-[#089981] flex items-center gap-1">
        <ShieldCheck class="w-3.5 h-3.5" /> 0-Penalty TF Guard
      </span>
      <span class="text-[#363a45]">•</span>
      <span class="text-[#2962ff]">🇨🇭 Dukascopy Swiss ECN</span>
      <span class="text-[#363a45]">•</span>
      <span class="text-[#d1d4dc]">120 FPS WebGL</span>
    </div>
  </div>

  <!-- Right: Quick Hub Actions -->
  <div class="flex items-center gap-2">
    {#if onOpenModelHub}
      <button
        onclick={onOpenModelHub}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-mono font-bold bg-[#1e222d] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/50 text-[#d1d4dc] hover:text-white transition-colors shadow-sm"
        title="Multi-Model Comparison Matrix"
      >
        <Layers class="w-3.5 h-3.5 text-[#2962ff]" />
        <span>Model Hub</span>
      </button>
    {/if}

    {#if onOpenMonteCarlo}
      <button
        onclick={onOpenMonteCarlo}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-mono font-bold bg-[#1e222d] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/50 text-[#d1d4dc] hover:text-white transition-colors shadow-sm"
        title="Monte Carlo Stress Testing"
      >
        <Dices class="w-3.5 h-3.5 text-[#f5c344]" />
        <span>Stress Test</span>
      </button>
    {/if}
  </div>
</header>

