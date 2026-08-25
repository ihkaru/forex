<script lang="ts">
  import {
    Search,
    Star,
    Info,
    Settings,
    X,
    Check,
    Sliders,
    Zap,
    Clock,
    BarChart3,
    Cpu,
    ShieldCheck,
    Layers,
    Plus
  } from '@lucide/svelte';
  import type { IChartLayer, ChartLayerContext } from '../../ports/layers';
  import { MarketSessionsLayer } from '../../adapters/layers/MarketSessionsLayer';

  import type { StrategyDescriptor } from '../../ports';

  interface Props {
    isOpen: boolean;
    layers: IChartLayer[];
    strategies?: StrategyDescriptor[];
    activeStrategyId?: string;
    layerContext: ChartLayerContext | null;
    onToggleLayer: (layerId: string) => void;
    onClose: () => void;
  }

  let {
    isOpen = false,
    layers = [],
    strategies = [],
    activeStrategyId = 'pola-n-v3',
    layerContext = null,
    onToggleLayer,
    onClose
  }: Props = $props();


  let searchQuery = $state('');
  let selectedCategory = $state<'ALL' | 'FAVORITES' | 'STRATEGIES' | 'TECHNICALS' | 'SESSIONS' | 'EXECUTION' | 'ACTIVE'>('ALL');
  let favorites = $state<Record<string, boolean>>({
    'volume-liquidity': true,
    'market-sessions': true,
    'dual-ema': true,
    'strat-pola-n-v2': true,
  });
  let activeConfigLayerId = $state<string | null>(null);
  let activeInfoId = $state<string | null>(null);

  interface CatalogItem {
    id: string;
    layer?: IChartLayer;
    name: string;
    typeTag: 'Strategy' | 'Indicator' | 'Session' | 'Execution';
    category: 'STRATEGIES' | 'TECHNICALS' | 'SESSIONS' | 'EXECUTION';
    author: string;
    description: string;
    isActive: boolean;
    hasSettings: boolean;
  }

  let catalog = $derived.by<CatalogItem[]>(() => {
    const items: CatalogItem[] = [];

    // Registered Layers
    layers.forEach((l) => {
      let typeTag: 'Strategy' | 'Indicator' | 'Session' | 'Execution' = 'Indicator';
      let category: 'STRATEGIES' | 'TECHNICALS' | 'SESSIONS' | 'EXECUTION' = 'TECHNICALS';
      let author = 'Built-in';
      let hasSettings = false;

      if (l.id === 'market-sessions') {
        typeTag = 'Session';
        category = 'SESSIONS';
        author = 'ECN Flow';
        hasSettings = true;
      } else if (l.id === 'volume-liquidity') {
        typeTag = 'Indicator';
        category = 'TECHNICALS';
        author = 'Dukascopy';
        hasSettings = false;
      } else if (l.id === 'backtest-trades') {
        typeTag = 'Execution';
        category = 'EXECUTION';
        author = 'Zero-Lookahead';
        hasSettings = false;
      } else if (l.id === 'active-signal') {
        typeTag = 'Execution';
        category = 'EXECUTION';
        author = 'TF Guard';
        hasSettings = false;
      } else if (l.id === 'pola-n-swings') {
        typeTag = 'Indicator';
        category = 'TECHNICALS';
        author = 'Fractal (4,3)';
        hasSettings = false;
      } else if (l.id === 'dual-ema') {
        typeTag = 'Indicator';
        category = 'TECHNICALS';
        author = 'Trend (20/50)';
        hasSettings = false;
      } else if (l.id === 'ict-order-blocks') {
        typeTag = 'Strategy';
        category = 'STRATEGIES';
        author = 'SMC Engine';
        hasSettings = false;
      }

      items.push({
        id: l.id,
        layer: l,
        name: l.name,
        typeTag,
        category,
        author,
        description: l.description,
        isActive: l.visible,
        hasSettings,
      });
    });

    // Dynamic strategy rule engines from registry
    if (strategies && strategies.length > 0) {
      for (const s of strategies) {
        items.push({
          id: `strat-${s.id}`,
          name: s.name,
          typeTag: 'Strategy',
          category: 'STRATEGIES',
          author: s.author || 'TF Quant',
          description: s.description,
          isActive: s.id === activeStrategyId,
          hasSettings: (s.parameters && s.parameters.length > 0) || false,
        });
      }
    }


    return items;
  });

  let filteredItems = $derived.by<CatalogItem[]>(() => {
    let list = catalog;

    if (selectedCategory === 'FAVORITES') {
      list = list.filter((i) => favorites[i.id]);
    } else if (selectedCategory === 'ACTIVE') {
      list = list.filter((i) => i.isActive);
    } else if (selectedCategory !== 'ALL') {
      list = list.filter((i) => i.category === selectedCategory);
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      list = list.filter(
        (item) =>
          item.name.toLowerCase().includes(q) ||
          item.typeTag.toLowerCase().includes(q) ||
          item.author.toLowerCase().includes(q)
      );
    }

    return list;
  });

  let counts = $derived.by(() => {
    return {
      ALL: catalog.length,
      FAVORITES: catalog.filter((i) => favorites[i.id]).length,
      STRATEGIES: catalog.filter((i) => i.category === 'STRATEGIES').length,
      TECHNICALS: catalog.filter((i) => i.category === 'TECHNICALS').length,
      SESSIONS: catalog.filter((i) => i.category === 'SESSIONS').length,
      EXECUTION: catalog.filter((i) => i.category === 'EXECUTION').length,
      ACTIVE: catalog.filter((i) => i.isActive).length,
    };
  });

  function toggleFavorite(id: string, e: MouseEvent) {
    e.stopPropagation();
    favorites[id] = !favorites[id];
  }

  function toggleSettings(id: string, e: MouseEvent) {
    e.stopPropagation();
    activeConfigLayerId = activeConfigLayerId === id ? null : id;
  }

  function toggleInfo(id: string, e: MouseEvent) {
    e.stopPropagation();
    activeInfoId = activeInfoId === id ? null : id;
  }

  function handleItemClick(item: CatalogItem) {
    if (item.layer) {
      onToggleLayer(item.layer.id);
    }
  }

  function handleReRender() {
    if (layerContext) {
      const activeLayer = layers.find((l) => l.id === activeConfigLayerId);
      if (activeLayer && activeLayer.visible) {
        activeLayer.render(layerContext);
      }
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-100 font-sans">
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl w-full max-w-3xl h-[540px] max-h-[90vh] flex flex-col shadow-2xl overflow-hidden animate-in zoom-in-95 duration-100">
      <!-- Minimalist Header with Clean Search -->
      <div class="px-4 py-3 border-b border-[#2a2e39] bg-[#1e222d] flex items-center justify-between gap-4">
        <h2 class="text-sm font-bold text-white font-mono whitespace-nowrap">
          Indicators, Metrics & Strategies
        </h2>

        <!-- Compact Search Input (TradingView Style) -->
        <div class="relative flex-1 max-w-md">
          <Search class="w-4 h-4 text-[#787b86] absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search..."
            class="w-full pl-9 pr-8 py-1.5 bg-[#131722] border border-[#2a2e39] focus:border-[#2962ff] rounded-lg text-xs font-mono text-white placeholder-[#787b86] focus:outline-none transition-all"
          />
          {#if searchQuery}
            <button
              onclick={() => searchQuery = ''}
              class="absolute right-2.5 top-1/2 -translate-y-1/2 p-0.5 text-[#787b86] hover:text-white"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          {/if}
        </div>

        <button
          onclick={onClose}
          class="p-1 rounded-lg text-[#787b86] hover:text-white hover:bg-[#2a2e39] transition-all"
          title="Close (Esc)"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Main Layout: Compact Sidebar + Clean Item Rows -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Sidebar Navigation -->
        <div class="w-52 border-r border-[#2a2e39] bg-[#131722]/40 p-2 space-y-0.5 overflow-y-auto font-mono text-xs select-none flex-shrink-0">
          <button
            onclick={() => selectedCategory = 'FAVORITES'}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-all {selectedCategory === 'FAVORITES' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#2a2e39]'}"
          >
            <div class="flex items-center gap-2">
              <Star class="w-3.5 h-3.5 text-[#f5c344] fill-[#f5c344]" />
              <span>Favorites</span>
            </div>
            <span class="text-[10px] opacity-60">{counts.FAVORITES}</span>
          </button>

          <button
            onclick={() => selectedCategory = 'ALL'}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-all {selectedCategory === 'ALL' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#2a2e39]'}"
          >
            <div class="flex items-center gap-2">
              <Layers class="w-3.5 h-3.5" />
              <span>All</span>
            </div>
            <span class="text-[10px] opacity-60">{counts.ALL}</span>
          </button>

          <button
            onclick={() => selectedCategory = 'TECHNICALS'}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-all {selectedCategory === 'TECHNICALS' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#2a2e39]'}"
          >
            <div class="flex items-center gap-2">
              <BarChart3 class="w-3.5 h-3.5 text-[#089981]" />
              <span>Technicals</span>
            </div>
            <span class="text-[10px] opacity-60">{counts.TECHNICALS}</span>
          </button>

          <button
            onclick={() => selectedCategory = 'STRATEGIES'}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-all {selectedCategory === 'STRATEGIES' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#2a2e39]'}"
          >
            <div class="flex items-center gap-2">
              <Cpu class="w-3.5 h-3.5 text-[#f5c344]" />
              <span>Strategies</span>
            </div>
            <span class="text-[10px] opacity-60">{counts.STRATEGIES}</span>
          </button>

          <button
            onclick={() => selectedCategory = 'SESSIONS'}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-all {selectedCategory === 'SESSIONS' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#2a2e39]'}"
          >
            <div class="flex items-center gap-2">
              <Clock class="w-3.5 h-3.5 text-[#818cf8]" />
              <span>Sessions</span>
            </div>
            <span class="text-[10px] opacity-60">{counts.SESSIONS}</span>
          </button>

          <button
            onclick={() => selectedCategory = 'EXECUTION'}
            class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-all {selectedCategory === 'EXECUTION' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#2a2e39]'}"
          >
            <div class="flex items-center gap-2">
              <ShieldCheck class="w-3.5 h-3.5 text-[#00E676]" />
              <span>Execution</span>
            </div>
            <span class="text-[10px] opacity-60">{counts.EXECUTION}</span>
          </button>

          <div class="pt-1.5 mt-1.5 border-t border-[#2a2e39]/60">
            <button
              onclick={() => selectedCategory = 'ACTIVE'}
              class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-all {selectedCategory === 'ACTIVE' ? 'bg-[#089981] text-white font-bold' : 'text-[#089981] hover:bg-[#2a2e39]'}"
            >
              <div class="flex items-center gap-2">
                <Check class="w-3.5 h-3.5" />
                <span>Active on Chart</span>
              </div>
              <span class="text-[10px] font-bold">{counts.ACTIVE}</span>
            </button>
          </div>
        </div>

        <!-- Clean, Minimalist Rows List (TradingView Native Style) -->
        <div class="flex-1 overflow-y-auto divide-y divide-[#2a2e39]/40 font-mono text-xs">
          {#if filteredItems.length === 0}
            <div class="h-48 flex flex-col items-center justify-center text-[#787b86]">
              <Search class="w-6 h-6 mb-1.5 opacity-40 text-[#2962ff]" />
              <p class="text-xs">No matching indicators or strategies found</p>
            </div>
          {:else}
            {#each filteredItems as item}
              <div class="flex flex-col">
                <!-- Single Row Item -->
                <div
                  role="button"
                  tabindex="0"
                  onclick={() => handleItemClick(item)}
                  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleItemClick(item); } }}
                  class="flex items-center justify-between px-4 py-2.5 hover:bg-[#2a2e39]/50 cursor-pointer transition-colors group select-none {item.isActive ? 'bg-[#2962ff]/5' : ''}"
                >
                  <!-- Left: Star + Name + Type Tag -->
                  <div class="flex items-center gap-2.5 min-w-0 flex-1">
                    <button
                      onclick={(e) => toggleFavorite(item.id, e)}
                      class="p-0.5 text-[#787b86] hover:text-[#f5c344] transition-colors"
                      title={favorites[item.id] ? 'Remove from favorites' : 'Add to favorites'}
                    >
                      <Star class="w-3.5 h-3.5 {favorites[item.id] ? 'text-[#f5c344] fill-[#f5c344]' : 'opacity-40 group-hover:opacity-100'}" />
                    </button>

                    <span class="font-bold text-white group-hover:text-[#2962ff] transition-colors truncate">
                      {item.name}
                    </span>

                    <span class="text-[9px] px-1.5 py-0.2 rounded font-mono {item.typeTag === 'Strategy' ? 'bg-[#f5c344]/15 text-[#f5c344]' : item.typeTag === 'Session' ? 'bg-[#818cf8]/15 text-[#818cf8]' : item.typeTag === 'Execution' ? 'bg-[#00E676]/15 text-[#00E676]' : 'bg-[#2a2e39] text-[#787b86]'}">
                      {item.typeTag}
                    </span>

                    <span class="text-[10px] text-[#787b86] hidden sm:inline truncate">
                      {item.author}
                    </span>
                  </div>

                  <!-- Right: Actions (Info, Settings, Status) -->
                  <div class="flex items-center gap-1.5 flex-shrink-0">
                    <!-- Info Button -->
                    <button
                      onclick={(e) => toggleInfo(item.id, e)}
                      class="p-1 rounded text-[#787b86] hover:text-white hover:bg-[#2a2e39] transition-all"
                      title="Show details"
                    >
                      <Info class="w-3.5 h-3.5" />
                    </button>

                    <!-- Settings Button if configurable -->
                    {#if item.hasSettings}
                      <button
                        onclick={(e) => toggleSettings(item.id, e)}
                        class="p-1 rounded text-[#787b86] hover:text-white hover:bg-[#2a2e39] transition-all {activeConfigLayerId === item.id ? 'text-[#2962ff] bg-[#2962ff]/10' : ''}"
                        title="Settings"
                      >
                        <Settings class="w-3.5 h-3.5" />
                      </button>
                    {/if}

                    <!-- Active / Apply Indicator Pill -->
                    {#if item.isActive}
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-[#089981]/20 text-[#089981] flex items-center gap-1">
                        <Check class="w-3 h-3" />
                        <span>Active</span>
                      </span>
                    {:else}
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold text-[#787b86] group-hover:text-white group-hover:bg-[#2a2e39] transition-all flex items-center gap-1">
                        <Plus class="w-3 h-3" />
                        <span>Add</span>
                      </span>
                    {/if}
                  </div>
                </div>

                <!-- Inline Info Popover Description -->
                {#if activeInfoId === item.id}
                  <div class="px-4 py-2 bg-[#131722] text-[11px] text-[#787b86] border-t border-[#2a2e39]/50 animate-in fade-in duration-75">
                    {item.description}
                  </div>
                {/if}

                <!-- Inline Settings Drawer for MarketSessionsLayer -->
                {#if activeConfigLayerId === item.id && item.layer instanceof MarketSessionsLayer}
                  {@const sessionLayer = item.layer}
                  <div class="px-4 py-3 bg-[#131722] border-t border-[#2a2e39] space-y-2 text-xs animate-in fade-in duration-75">
                    <div class="flex items-center justify-between font-bold text-[#d1d4dc]">
                      <span class="flex items-center gap-1.5">
                        <Sliders class="w-3.5 h-3.5 text-[#2962ff]" />
                        <span>Session Visibility & Intensity</span>
                      </span>
                      <span class="text-[10px] text-[#089981]">Auto-applied</span>
                    </div>

                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-[11px]">
                      <label class="flex items-center gap-1.5 p-1.5 rounded bg-[#1e222d] border border-[#2a2e39] cursor-pointer">
                        <input
                          type="checkbox"
                          bind:checked={sessionLayer.config.showAsia}
                          onchange={handleReRender}
                          class="rounded bg-[#131722] border-[#2a2e39] text-[#2962ff]"
                        />
                        <span class="text-[#818cf8]">Tokyo (00-08)</span>
                      </label>

                      <label class="flex items-center gap-1.5 p-1.5 rounded bg-[#1e222d] border border-[#2a2e39] cursor-pointer">
                        <input
                          type="checkbox"
                          bind:checked={sessionLayer.config.showLondon}
                          onchange={handleReRender}
                          class="rounded bg-[#131722] border-[#2a2e39] text-[#2962ff]"
                        />
                        <span class="text-[#06b6d4]">London (07-13)</span>
                      </label>

                      <label class="flex items-center gap-1.5 p-1.5 rounded bg-[#1e222d] border border-[#2a2e39] cursor-pointer">
                        <input
                          type="checkbox"
                          bind:checked={sessionLayer.config.showOverlap}
                          onchange={handleReRender}
                          class="rounded bg-[#1e222d] border-[#2a2e39] text-[#2962ff]"
                        />
                        <span class="text-[#10b981]">Overlap (13-16)</span>
                      </label>

                      <label class="flex items-center gap-1.5 p-1.5 rounded bg-[#1e222d] border border-[#2a2e39] cursor-pointer">
                        <input
                          type="checkbox"
                          bind:checked={sessionLayer.config.showNewYork}
                          onchange={handleReRender}
                          class="rounded bg-[#1e222d] border-[#2a2e39] text-[#2962ff]"
                        />
                        <span class="text-[#f59e0b]">NY (16-21)</span>
                      </label>
                    </div>

                    <!-- Opacity Slider -->
                    <div class="flex items-center gap-3 pt-1">
                      <span class="text-[10px] text-[#787b86]">Opacity:</span>
                      <input
                        type="range"
                        min="0.2"
                        max="2.5"
                        step="0.1"
                        bind:value={sessionLayer.config.opacity}
                        oninput={handleReRender}
                        class="flex-1 h-1.5 bg-[#2a2e39] rounded-lg appearance-none cursor-pointer accent-[#2962ff]"
                      />
                      <span class="text-[10px] font-bold text-[#2962ff]">{(sessionLayer.config.opacity * 100).toFixed(0)}%</span>
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
