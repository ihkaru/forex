<script lang="ts">
  import {
    Cpu,
    Search,
    Dices,
    ChevronDown,
    Check,
    Zap,
    Layers,
    CheckCircle2
  } from '@lucide/svelte';
  import type { StrategyDescriptor } from '../ports';

  interface Props {
    strategies: StrategyDescriptor[];
    selectedStrategyId: string;
    onSelectStrategy: (strategyId: string) => void;
    onOpenModelHub: () => void;
    onOpenMonteCarlo: () => void;
  }

  let {
    strategies = [],
    selectedStrategyId = 'pola-n-core',
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

  function handleSelect(id: string) {
    onSelectStrategy(id);
    isDropdownOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isDropdownOpen = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- TradingView Native Header Strategy Strip -->
<div class="flex flex-col gap-2 bg-[#1e222d] px-4 py-2.5 rounded-xl border border-[#2a2e39] relative z-20">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <!-- Left: Strategy Selector Dropdown (TradingView Style) -->
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-1.5 text-xs font-bold text-[#787b86] font-mono">
        <Cpu class="w-4 h-4 text-[#2962ff]" />
        <span>Strategy Engine:</span>
      </div>

      <!-- TradingView-Inspired Strategy Selector Button -->
      <div class="relative">
        <button
          onclick={() => isDropdownOpen = !isDropdownOpen}
          class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-mono font-bold bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-white transition-all shadow-sm group"
        >
          <!-- fx / Zap Symbol -->
          <span class="px-1.5 py-0.5 rounded bg-[#2962ff]/20 text-[#2962ff] font-extrabold text-[10px] flex items-center gap-1">
            <Zap class="w-3 h-3 text-[#2962ff]" /> fx
          </span>

          <span class="font-bold text-[#d1d4dc] group-hover:text-white transition-colors">
            {selectedStrategy?.name ?? 'Pilih Strategi...'}
          </span>

          {#if isGoldSpecialist}
            <span class="text-[9px] px-1.5 py-0.5 rounded font-extrabold bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40 font-mono">
              ⭐ GOLD SPECIALIST
            </span>
          {:else if selectedStrategy?.category === 'MARKET_STRUCTURE'}
            <span class="text-[9px] px-1.5 py-0.5 rounded font-extrabold bg-[#2962ff]/20 text-[#2962ff] font-mono">
              🌐 FX MAJOR
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
          <!-- Backdrop to close on click outside -->
          <div
            class="fixed inset-0 z-30"
            onclick={() => isDropdownOpen = false}
            role="presentation"
          ></div>

          <div class="absolute left-0 top-full mt-1.5 w-80 bg-[#1e222d] border border-[#2a2e39] rounded-xl shadow-2xl z-40 overflow-hidden animate-in fade-in zoom-in-95 duration-150">
            <div class="p-2 border-b border-[#2a2e39] bg-[#131722]/80 flex items-center justify-between text-[11px] font-mono text-[#787b86]">
              <span class="font-bold text-[#d1d4dc]">PILIH QUANTITATIVE MODEL</span>
              <span>{strategies.length} Model</span>
            </div>

            <div class="p-1.5 max-h-64 overflow-y-auto space-y-1">
              {#each strategies as strat}
                {@const isSelected = selectedStrategyId === strat.id}
                {@const isGold = strat.isSpecialist || strat.id === 'pola-n-v2'}
                <button
                  onclick={() => handleSelect(strat.id)}
                  class="w-full flex items-center justify-between p-2 rounded-lg text-xs font-mono text-left transition-all {isSelected ? 'bg-[#2962ff] text-white shadow-sm' : 'text-[#d1d4dc] hover:bg-[#131722] hover:text-white'}"
                >
                  <div class="flex flex-col gap-0.5">
                    <div class="flex items-center gap-1.5">
                      <span class="font-bold">{strat.name}</span>
                      {#if isGold}
                        <span class="text-[8px] px-1 py-0.2 rounded font-extrabold {isSelected ? 'bg-[#f5c344] text-black' : 'bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40'}">
                          ⭐ GOLD
                        </span>
                      {/if}
                    </div>
                    <span class="text-[10px] {isSelected ? 'text-white/80' : 'text-[#787b86]'}">
                      PF {strat.profitFactor.toFixed(2)} • WR {strat.winRatePct.toFixed(1)}% • RF {strat.recoveryFactor.toFixed(2)}
                    </span>
                  </div>

                  {#if isSelected}
                    <Check class="w-4 h-4 text-white flex-shrink-0" />
                  {/if}
                </button>
              {/each}
            </div>

            <div class="p-2 border-t border-[#2a2e39] bg-[#131722]/90">
              <button
                onclick={() => {
                  isDropdownOpen = false;
                  onOpenModelHub();
                }}
                class="w-full flex items-center justify-center gap-1.5 py-1.5 px-3 rounded-lg bg-[#2962ff]/15 hover:bg-[#2962ff]/25 text-[#2962ff] border border-[#2962ff]/40 text-xs font-mono font-bold transition-all shadow-sm"
              >
                <Search class="w-3.5 h-3.5" /> Buka Model Hub Lengkap ({strategies.length})
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Right: TradingView Utility Actions -->
    <div class="flex items-center gap-2">
      <!-- Open Full Model Hub (TradingView 'fx Indicators' Modal Style) -->
      <button
        onclick={onOpenModelHub}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#2962ff]/15 hover:bg-[#2962ff]/25 text-[#2962ff] border border-[#2962ff]/40 text-xs font-mono font-bold transition-all shadow-sm"
        title="Open Full Quantitative Engine Registry & 6-Pillar Scorecard"
      >
        <Search class="w-3.5 h-3.5" />
        <span>Model Hub ({strategies.length})</span>
      </button>

      <!-- Quick Monte Carlo Trigger -->
      <button
        onclick={onOpenMonteCarlo}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#ab47bc]/20 hover:bg-[#ab47bc]/30 text-[#ab47bc] border border-[#ab47bc]/40 text-xs font-mono font-bold transition-all"
        title="Monte Carlo 1,000-Path Resampling & Risk of Ruin"
      >
        <Dices class="w-3.5 h-3.5" />
        <span>Monte Carlo</span>
      </button>
    </div>
  </div>

  <!-- Active Strategy Parameters Ribbon (TradingView Legend Style) -->
  <div class="flex items-center gap-2 text-[10px] text-[#787b86] font-mono pt-1.5 border-t border-[#2a2e39]/60 overflow-x-auto whitespace-nowrap">
    <span class="text-[#2962ff] font-bold flex items-center gap-1 flex-shrink-0">
      <Zap class="w-3 h-3 text-[#2962ff]" /> Active Parameters:
    </span>

    {#if selectedStrategyId === 'pola-n-core'}
      <span class="text-[#d1d4dc]">Swing (5, 3) • Retest Golden Pocket 61.8% • Session 10:00–21:00 UTC (NY/London) • Target R:R 1:1.30 • Breakeven @ 30% MFE</span>
    {:else if selectedStrategyId === 'pola-n-v2'}
      <span class="text-[#f5c344] font-semibold">⭐ Gold Specialist: Swing (5, 3) • Golden Pocket 61.8% Limit • Session 10:00–21:00 UTC Overlap • Target R:R 1:1.20 • Breakeven @ 30% MFE</span>
    {:else if selectedStrategyId === 'dual-ema-trend'}
      <span class="text-[#d1d4dc]">EMA Fast (12) &gt; Slow (36) Cross • Slope Momentum Filter • Target R:R 1:1.50 • Trend Expansion</span>
    {:else}
      <span class="text-[#d1d4dc]">ICT Order Block Sweep • FVG Liquidity Mitigation • Target R:R 1:2.00 • Institutional SMC</span>
    {/if}
  </div>
</div>
