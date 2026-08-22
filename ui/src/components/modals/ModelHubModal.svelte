<script lang="ts">
  import { Search, Check, CheckCircle2, Zap, Cpu } from '@lucide/svelte';
  import type { StrategyDescriptor } from '../../ports';

  interface Props {
    strategies: StrategyDescriptor[];
    selectedStrategyId?: string;
    onSelectStrategy?: (strategyId: string) => void;
    onClose: () => void;
  }

  let {
    strategies = [],
    selectedStrategyId = 'pola-n-core',
    onSelectStrategy,
    onClose
  }: Props = $props();

  let stratSearchQuery = $state('');
  let stratCategoryFilter = $state('ALL');
  let stratSortBy = $state<'pf' | 'winRate' | 'rf' | 'wfer'>('pf');

  let filteredStrategies = $derived(
    strategies
      .filter((s) => {
        const query = stratSearchQuery.trim().toLowerCase();
        const matchesSearch =
          !query ||
          s.name.toLowerCase().includes(query) ||
          s.code.toLowerCase().includes(query) ||
          s.description.toLowerCase().includes(query) ||
          s.category.toLowerCase().includes(query);
        const matchesCategory =
          stratCategoryFilter === 'ALL' ||
          (stratCategoryFilter === 'SPECIALIST' && s.isSpecialist) ||
          s.category === stratCategoryFilter;
        return matchesSearch && matchesCategory;
      })
      .sort((a, b) => {
        if (stratSortBy === 'pf') return b.profitFactor - a.profitFactor;
        if (stratSortBy === 'winRate') return b.winRatePct - a.winRatePct;
        if (stratSortBy === 'rf') return b.recoveryFactor - a.recoveryFactor;
        if (stratSortBy === 'wfer') return b.wferPct - a.wferPct;
        return 0;
      })
  );
</script>

<div class="flex flex-col gap-4">
  <!-- Header Description & Key Stats -->
  <div class="flex flex-wrap items-center justify-between gap-3 bg-[#131722] p-3.5 rounded-xl border border-[#2a2e39]">
    <div class="flex items-center gap-2">
      <Cpu class="w-5 h-5 text-[#2962ff]" />
      <div>
        <h3 class="text-xs font-bold text-white font-mono uppercase">Quantitative Engine Registry</h3>
        <p class="text-[11px] text-[#787b86]">Eksplorasi, bandingkan, dan aktifkan model kuantitatif terverifikasi Traders Family.</p>
      </div>
    </div>
    <div class="flex items-center gap-2 text-xs font-mono">
      <span class="px-2 py-1 rounded bg-[#2962ff]/20 text-[#2962ff] font-bold">{strategies.length} Models Loaded</span>
      <span class="px-2 py-1 rounded bg-[#089981]/20 text-[#089981] font-bold">100% TF Compliant</span>
    </div>
  </div>

  <!-- Toolbar: Search, Category Filters, and Sorting -->
  <div class="flex flex-col md:flex-row items-stretch md:items-center justify-between gap-3 bg-[#131722]/60 p-2.5 rounded-xl border border-[#2a2e39]">
    <!-- Search Bar -->
    <div class="relative flex-1">
      <Search class="w-4 h-4 text-[#787b86] absolute left-3 top-1/2 -translate-y-1/2" />
      <input
        type="text"
        bind:value={stratSearchQuery}
        placeholder="Cari strategi berdasarkan nama, kode, indikator (e.g. Pola N, EMA, FVG)..."
        class="w-full bg-[#1e222d] border border-[#2a2e39] focus:border-[#2962ff] rounded-lg pl-9 pr-3 py-1.5 text-xs text-white placeholder-[#787b86] outline-none font-mono transition-all"
      />
    </div>

    <!-- Category Filter Tabs -->
    <div class="flex flex-wrap items-center gap-1 text-[11px] font-mono">
      {#each ['ALL', 'SPECIALIST', 'MARKET_STRUCTURE', 'TREND_FOLLOWING', 'SMART_MONEY'] as cat}
        <button
          onclick={() => stratCategoryFilter = cat}
          class="px-2.5 py-1 rounded-lg font-bold transition-all {stratCategoryFilter === cat ? 'bg-[#2962ff] text-white' : 'bg-[#1e222d] text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]'}"
        >
          {#if cat === 'ALL'}Semua
          {:else if cat === 'SPECIALIST'}⭐ Specialist
          {:else if cat === 'MARKET_STRUCTURE'}Structure
          {:else if cat === 'TREND_FOLLOWING'}Trend
          {:else if cat === 'SMART_MONEY'}SMC / Liquidity
          {/if}
        </button>
      {/each}
    </div>

    <!-- Sorting Dropdown -->
    <div class="flex items-center gap-1.5 text-xs font-mono">
      <span class="text-[#787b86] text-[10px]">Sort:</span>
      <select
        bind:value={stratSortBy}
        class="bg-[#1e222d] border border-[#2a2e39] rounded-lg px-2 py-1 text-xs text-[#d1d4dc] font-mono outline-none cursor-pointer hover:border-[#2962ff]"
      >
        <option value="pf">Profit Factor (Tertinggi)</option>
        <option value="winRate">Win Rate (Tertinggi)</option>
        <option value="rf">Recovery Factor (Tertinggi)</option>
        <option value="wfer">WFER Stability (Tertinggi)</option>
      </select>
    </div>
  </div>

  <!-- Model Cards Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
    {#each filteredStrategies as strat}
      {@const isActive = selectedStrategyId === strat.id}
      <div class="p-4 rounded-xl transition-all {isActive ? 'bg-[#131722] border-2 border-[#2962ff] shadow-lg shadow-[#2962ff]/10' : 'bg-[#131722] border border-[#2a2e39] hover:border-[#787b86]/50'} flex flex-col justify-between">
        <div>
          <!-- Card Top Header -->
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-1.5">
              {#if strat.isSpecialist}
                <span class="text-[9px] px-1.5 py-0.5 rounded font-mono font-extrabold bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40">
                  ⭐ GOLD SPECIALIST
                </span>
              {:else}
                <span class="text-[9px] px-1.5 py-0.5 rounded font-mono font-bold bg-[#2962ff]/20 text-[#2962ff]">
                  {strat.category}
                </span>
              {/if}
              <span class="text-[9px] px-1.5 py-0.5 rounded font-mono bg-[#2a2e39] text-[#787b86]">
                {strat.code}
              </span>
            </div>

            {#if isActive}
              <span class="text-[9px] px-2 py-0.5 rounded-full font-mono font-extrabold bg-[#2962ff] text-white flex items-center gap-1 shadow-sm">
                <Check class="w-3 h-3" /> ACTIVE ENGINE
              </span>
            {:else}
              <span class="text-[9px] text-[#089981] font-mono font-bold flex items-center gap-1">
                <CheckCircle2 class="w-3 h-3" /> TF Compliant
              </span>
            {/if}
          </div>

          <!-- Title & Description -->
          <h3 class="text-sm font-bold text-white mb-1">{strat.name}</h3>
          <p class="text-[11px] text-[#787b86] leading-relaxed mb-3">{strat.description}</p>
        </div>

        <!-- 6-Factor Quant Metrics Grid -->
        <div>
          <div class="grid grid-cols-3 gap-2 py-2.5 border-t border-[#2a2e39] font-mono text-[10px] bg-[#1e222d]/60 rounded-lg px-3 mb-3">
            <div>
              <span class="text-[#787b86] block text-[9px]">WIN RATE</span>
              <span class="font-bold text-[#089981] text-xs">{strat.winRatePct.toFixed(1)}%</span>
            </div>
            <div>
              <span class="text-[#787b86] block text-[9px]">PROFIT FACTOR</span>
              <span class="font-bold text-white text-xs">{strat.profitFactor.toFixed(2)}</span>
            </div>
            <div>
              <span class="text-[#787b86] block text-[9px]">REC FACTOR</span>
              <span class="font-bold text-[#089981] text-xs">{strat.recoveryFactor.toFixed(2)}</span>
            </div>
            <div>
              <span class="text-[#787b86] block text-[9px]">SHARPE</span>
              <span class="font-bold text-[#f5c344]">{strat.sharpeRatio.toFixed(2)}</span>
            </div>
            <div>
              <span class="text-[#787b86] block text-[9px]">SORTINO</span>
              <span class="font-bold text-white">{strat.sortinoRatio.toFixed(2)}</span>
            </div>
            <div>
              <span class="text-[#787b86] block text-[9px]">WFER STABILITY</span>
              <span class="font-bold text-[#2962ff]">{strat.wferPct.toFixed(1)}%</span>
            </div>
          </div>

          <!-- Card Actions -->
          <div class="flex items-center justify-between gap-2 pt-1 border-t border-[#2a2e39]/60">
            <div class="text-[10px] text-[#787b86] font-mono">
              Instrumen: <span class="text-white font-bold">{strat.supportedSymbols && strat.supportedSymbols.length === 1 ? 'XAU/USD Exclusive' : 'Multi-Asset (8 Pairs)'}</span>
            </div>

            <button
              onclick={() => {
                if (onSelectStrategy) {
                  onSelectStrategy(strat.id);
                }
                onClose();
              }}
              disabled={isActive}
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-bold transition-all {isActive ? 'bg-[#2962ff]/20 text-[#2962ff] cursor-default' : 'bg-[#2962ff] hover:bg-[#1e4bd8] text-white shadow-sm'}"
            >
              {#if isActive}
                <Check class="w-3.5 h-3.5" /> Sedang Aktif
              {:else}
                <Zap class="w-3.5 h-3.5" /> Aktifkan Model
              {/if}
            </button>
          </div>
        </div>
      </div>
    {/each}
  </div>

  {#if filteredStrategies.length === 0}
    <div class="p-8 text-center bg-[#131722] rounded-xl border border-[#2a2e39] text-[#787b86] font-mono text-xs">
      Tidak ada strategi yang cocok dengan pencarian "<strong>{stratSearchQuery}</strong>" pada kategori ini.
    </div>
  {/if}
</div>
