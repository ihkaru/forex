<script lang="ts">
  import {
    Info,
    RefreshCw,
    Layers,
    Sliders,
    Eye,
    EyeOff,
    Clock,
    ChevronDown,
    Check
  } from '@lucide/svelte';

  interface PairItem {
    symbol: string;
    ticker?: string;
    base: string;
    quote: string;
    tier: number;
    multiplier: number;
  }

  interface LayerItem {
    id: string;
    name: string;
    shortLabel?: string;
    visible: boolean;
    isUniversal?: boolean;
  }

  interface Props {
    activeSymbol: string;
    displayPairs: PairItem[];
    activePairs: PairItem[];
    supportedSymbols?: string[];
    isSpecialist?: boolean;
    selectedSource: 'dukascopy' | 'mrg_demo' | 'mrg_real';
    currentPrice: number;
    applicableLayers: LayerItem[];
    syncStatusMessage?: string | null;
    activeRange: string;
    userTimeZone: string;
    userTimeZoneCity: string;
    userTimeZoneOffset: string;
    onSelectSymbol: (symbol: string) => void;
    onSelectSource?: (source: 'dukascopy' | 'mrg_demo' | 'mrg_real') => void;
    onSyncDelta?: () => void;
    onOpenProvenance?: () => void;
    onOpenLayersModal: () => void;
    onToggleLayer: (layerId: string) => void;
    onZoom: (range: string) => void;
  }

  let {
    activeSymbol,
    displayPairs,
    activePairs,
    supportedSymbols = [],
    isSpecialist = false,
    selectedSource = 'mrg_demo',
    currentPrice,
    applicableLayers,
    syncStatusMessage = null,
    activeRange,
    userTimeZone,
    userTimeZoneCity,
    userTimeZoneOffset,
    onSelectSymbol,
    onSelectSource,
    onSyncDelta,
    onOpenProvenance,
    onOpenLayersModal,
    onToggleLayer,
    onZoom
  }: Props = $props();

  let isSymbolDropdownOpen = $state(false);
  let currentPair = $derived(activePairs.find(p => p.symbol === activeSymbol));
</script>

<div class="flex flex-wrap items-center justify-between gap-2.5 pb-2.5 border-b border-[#2a2e39] relative z-20">
  <!-- Left: Symbol Dropdown, Timeframe, Provenance Info + Sync Icon, Price & Spread -->
  <div class="flex flex-wrap items-center gap-2">
    <!-- TradingView Symbol Dropdown Selector -->
    <div class="relative">
      <button
        onclick={() => isSymbolDropdownOpen = !isSymbolDropdownOpen}
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-mono font-bold bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-white transition-all shadow-sm group"
      >
        <span class="text-sm">{activeSymbol.startsWith('XAU') ? '🪙' : '🇨🇭'}</span>
        <span class="text-white font-extrabold tracking-wide">
          {currentPair?.ticker || `${activeSymbol.slice(0, 3)}/${activeSymbol.slice(3)}`}
        </span>

        {#if isSpecialist || (supportedSymbols && supportedSymbols.length === 1)}
          <span class="text-[9px] px-1.5 py-0.5 rounded font-extrabold bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40 font-mono">
            ⭐ GOLD SPECIALIST
          </span>
        {:else}
          <span class="text-[9px] px-1.5 py-0.5 rounded font-bold bg-[#2a2e39] text-[#f5c344] font-mono">
            T{currentPair?.tier ?? 1} ({currentPair?.multiplier.toFixed(1) ?? '2.0'}x)
          </span>
        {/if}

        <ChevronDown class="w-3.5 h-3.5 text-[#787b86] group-hover:text-white transition-transform {isSymbolDropdownOpen ? 'rotate-180' : ''}" />
      </button>

      <!-- Symbol Picker Dropdown -->
      {#if isSymbolDropdownOpen}
        <div class="absolute left-0 top-full mt-1.5 w-64 bg-[#1e222d] border border-[#2a2e39] rounded-xl shadow-2xl p-1.5 z-50 flex flex-col gap-0.5 animate-in fade-in zoom-in-95 duration-100">
          <div class="px-2 py-1 text-[10px] font-mono font-bold text-[#787b86] uppercase tracking-wider border-b border-[#2a2e39]/50 mb-1 flex items-center justify-between">
            <span>TradingView Pairs</span>
            <span>VP Multiplier</span>
          </div>

          {#each displayPairs as pair}
            <button
              onclick={() => {
                onSelectSymbol(pair.symbol);
                isSymbolDropdownOpen = false;
              }}
              class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs font-mono text-left transition-all {activeSymbol === pair.symbol ? 'bg-[#2962ff] text-white font-bold' : 'text-[#d1d4dc] hover:bg-[#2a2e39]'}"
            >
              <div class="flex items-center gap-2">
                <span class="text-xs">{pair.symbol.startsWith('XAU') ? '🪙' : '🇨🇭'}</span>
                <span>{pair.ticker || `${pair.base}/${pair.quote}`}</span>
                {#if pair.symbol === 'XAUUSD'}
                  <span class="text-[9px] px-1 rounded bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/30 font-bold">Gold</span>
                {/if}
              </div>

              <div class="flex items-center gap-1.5">
                <span class="text-[10px] {activeSymbol === pair.symbol ? 'text-white/80' : 'text-[#787b86]'}">
                  Tier {pair.tier} ({pair.multiplier.toFixed(1)}x)
                </span>
                {#if activeSymbol === pair.symbol}
                  <Check class="w-3 h-3 text-white" />
                {/if}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Active Timeframe Pill -->
    <div class="px-2.5 py-1 rounded-lg text-xs font-mono font-extrabold bg-[#2962ff]/15 text-[#2962ff] border border-[#2962ff]/30 shadow-sm flex items-center gap-1">
      <span class="w-1.5 h-1.5 rounded-full bg-[#2962ff] animate-pulse"></span>
      <span>H1</span>
    </div>

    <!-- Data Source Triple Switcher (Dukascopy vs MT4 Live Demo vs MT4 Real) -->
    <div class="flex items-center bg-[#131722] p-0.5 rounded-lg border border-[#2a2e39] text-[10px] font-mono">
      <button
        onclick={() => onSelectSource?.('mrg_demo')}
        class="px-2 py-0.5 rounded font-bold transition-all {selectedSource === 'mrg_demo' ? 'bg-[#089981] text-white shadow-sm' : 'text-[#787b86] hover:text-white'}"
        title="Gunakan Data Live MT4 Broker MRG Demo (Zero Delay Live Feed)"
      >
        🟢 MRG DEMO
      </button>

      <button
        onclick={() => onSelectSource?.('dukascopy')}
        class="px-2 py-0.5 rounded font-bold transition-all {selectedSource === 'dukascopy' ? 'bg-[#2962ff] text-white shadow-sm' : 'text-[#787b86] hover:text-white'}"
        title="Gunakan Data Historis Dukascopy Bank SA (10 Tahun • Swiss True-Tick ECN)"
      >
        🇨🇭 DUKASCOPY (10Y)
      </button>

      <button
        onclick={() => onSelectSource?.('mrg_real')}
        class="px-2 py-0.5 rounded font-bold transition-all {selectedSource === 'mrg_real' ? 'bg-[#f23645] text-white shadow-sm' : 'text-[#787b86] hover:text-white'}"
        title="Gunakan Data Live MT4 Broker MRG Real (Live Market LP Feed)"
      >
        🔴 MRG REAL
      </button>
    </div>

    <!-- Provenance Info Icon Button -->
    <button
      onclick={onOpenProvenance}
      class="p-1.5 rounded-lg bg-[#131722] hover:bg-[#2a2e39] text-[#787b86] hover:text-white border border-[#2a2e39] transition-all shadow-sm"
      title="Buka Inspektur Asal-Usul & Provenance Data Pasar"
    >
      <Info class="w-3.5 h-3.5" />
    </button>

    <!-- Delta Sync Icon Trigger -->
    {#if onSyncDelta}
      <button
        onclick={onSyncDelta}
        class="p-1.5 rounded-lg bg-[#131722] hover:bg-[#2a2e39] text-[#00E676] hover:text-[#69f0ae] border border-[#2a2e39] hover:border-[#00E676]/50 transition-all shadow-sm"
        title="Trigger Continuous Delta Sync (High-Watermark Ingestion)"
      >
        <RefreshCw class="w-3 h-3" />
      </button>
    {/if}

    <!-- Live Price & Spread -->
    <div class="flex items-center gap-2 font-mono pl-1 border-l border-[#2a2e39]">
      <span class="text-sm font-black text-[#089981]">
        {currentPrice.toFixed(5)}
      </span>
      <span class="text-[10px] text-[#787b86]">0.8p spread</span>
    </div>
  </div>

  <!-- Center: Integrated Strategy-Adaptive Layer Toggles (TradingView Native Style) -->
  <div class="flex items-center gap-1 bg-[#131722] p-1 rounded-lg border border-[#2a2e39]">
    <button
      onclick={onOpenLayersModal}
      class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-mono font-bold text-[#787b86] hover:text-white hover:bg-[#2a2e39] border-r border-[#2a2e39] mr-0.5 transition-all"
      title="Buka Object Tree & Pengaturan Layer Lengkap"
    >
      <Layers class="w-3.5 h-3.5 text-[#2962ff]" />
      <Sliders class="w-3 h-3 text-[#787b86]" />
    </button>

    {#each applicableLayers as layer}
      <button
        onclick={() => onToggleLayer(layer.id)}
        title="{layer.name} ({layer.isUniversal ? 'Universal' : 'Strategy-Specific Indicator'})"
        class="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-mono font-semibold transition-all {layer.visible ? (layer.isUniversal ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/40') : 'bg-transparent text-[#787b86] opacity-60 border border-transparent'}"
      >
        {#if layer.visible}
          <Eye class="w-3 h-3 {layer.isUniversal ? 'text-[#089981]' : 'text-[#2962ff]'}" />
        {:else}
          <EyeOff class="w-3 h-3 text-[#787b86]" />
        {/if}
        <span>{layer.shortLabel ?? layer.name}</span>
      </button>
    {/each}
  </div>

  <!-- Right: Timezone/Volume Badge, Zoom Range Selectors & Live Sync Status -->
  <div class="flex items-center gap-2">
    <!-- Timezone & Liquidity Badge -->
    <div class="hidden sm:flex items-center gap-1.5 px-2 py-1 rounded-lg bg-[#131722] border border-[#2a2e39] text-[10px] font-mono select-none">
      <div class="flex items-center gap-1 text-[#2962ff] font-bold" title="Zona waktu lokal browser ({userTimeZone})">
        <Clock class="w-3 h-3 text-[#2962ff]" />
        <span>{userTimeZoneOffset} ({userTimeZoneCity})</span>
      </div>
      <span class="text-[#2a2e39] font-bold">|</span>
      <div class="flex items-center gap-1 text-[#787b86]" title="Indikator Volume Dukascopy ECN: Bar Neon = Lonjakan Likuiditas Besar (>1.5x SMA20)">
        <span class="w-1.5 h-1.5 rounded-sm bg-[#00f2fe]"></span>
        <span class="text-[9px] text-[#00f2fe] font-bold">High Vol</span>
        <span class="w-1.5 h-1.5 rounded-sm bg-[#089981]/60 ml-1"></span>
        <span class="text-[9px] text-[#787b86]">Norm</span>
      </div>
    </div>

    {#if syncStatusMessage}
      <div class="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-[#00E676]/10 text-[#00E676] text-xs font-mono border border-[#00E676]/30 animate-pulse shadow-sm">
        <span>{syncStatusMessage}</span>
      </div>
    {/if}

    <!-- Range Zoom Selectors -->
    <div class="flex items-center gap-0.5 bg-[#131722] p-1 rounded-lg border border-[#2a2e39]">
      {#each ['1W', '1M', '6M', '1Y', 'ALL'] as range}
        <button
          onclick={() => onZoom(range)}
          class="px-2 py-0.5 rounded text-[10px] font-mono font-bold transition-all {activeRange === range ? 'bg-[#2962ff] text-white shadow-sm' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
        >
          {range === 'ALL' ? 'ALL (10Y)' : range}
        </button>
      {/each}
    </div>
  </div>
</div>
