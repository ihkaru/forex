<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createChart,
    CandlestickSeries,
    type IChartApi,
    type ISeriesApi,
    type CandlestickData
  } from 'lightweight-charts';
  import {
    Eye,
    EyeOff,
    Layers,
    TrendingUp,
    GitBranch,
    ArrowUpDown,
    Zap,
    RefreshCw,
    Info,
    ChevronDown,
    Check
  } from '@lucide/svelte';
  import type { Candle, Signal } from '../domain/models';
  import type { SimulatedTrade } from '../ports/layers';
  import { ChartLayerManager } from '../adapters/layers/ChartLayerManager';

  interface Props {
    activeSymbol: string;
    activePairs: Array<{ symbol: string; base: string; quote: string; tier: number; multiplier: number }>;
    supportedSymbols?: string[];
    isSpecialist?: boolean;
    activeStrategyId?: string;
    activeStrategyCategory?: string;
    currentPrice: number;
    candles: Candle[];
    trades: SimulatedTrade[];
    signal: Signal | null;
    syncStatusMessage?: string | null;
    onSelectSymbol: (symbol: string) => void;
    onSyncDelta?: () => void;
    onOpenProvenance?: () => void;
  }

  let {
    activeSymbol = 'XAUUSD',
    activePairs = [],
    supportedSymbols = [],
    isSpecialist = false,
    activeStrategyId = 'pola-n-v2',
    activeStrategyCategory = 'GOLD_SPECIALIST',
    currentPrice = 0.85412,
    candles = [],
    trades = [],
    signal = null,
    syncStatusMessage = null,
    onSelectSymbol,
    onSyncDelta,
    onOpenProvenance
  }: Props = $props();

  let displayPairs = $derived(
    supportedSymbols && supportedSymbols.length > 0
      ? activePairs.filter((p) => supportedSymbols.includes(p.symbol))
      : activePairs
  );

  let isSymbolDropdownOpen = $state(false);
  let chartContainer: HTMLDivElement | null = $state(null);
  let chart: IChartApi | null = null;
  let candleSeries: ISeriesApi<'Candlestick'> | null = null;
  let activeRange = $state('1W');

  // Pure Composition Root: Chart Layer Manager
  const layerManager = new ChartLayerManager();
  let layersState = $state(layerManager.getAllLayers().map(l => ({
    id: l.id,
    name: l.name,
    visible: l.visible
  })));

  let applicableLayers = $derived(
    layerManager.getApplicableLayers(activeStrategyId, activeStrategyCategory)
  );

  function getContext() {
    return {
      chart: chart!,
      candleSeries: candleSeries!,
      candles,
      trades,
      signal,
      activeSymbol,
      activeStrategyId,
      activeStrategyCategory,
    };
  }

  function initChart() {
    if (!chartContainer) return;

    chart = createChart(chartContainer, {
      width: chartContainer.clientWidth,
      height: 500,
      layout: {
        background: { color: '#131722' },
        textColor: '#787b86',
      },
      grid: {
        vertLines: { color: '#1e222d' },
        horzLines: { color: '#1e222d' },
      },
      crosshair: {
        vertLine: {
          color: '#787b86',
          width: 1,
          style: 3,
        },
        horzLine: {
          color: '#787b86',
          width: 1,
          style: 3,
        },
      },
      timeScale: {
        borderColor: '#2a2e39',
        timeVisible: true,
        secondsVisible: false,
      },
      rightPriceScale: {
        borderColor: '#2a2e39',
      },
    });

    candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#089981',
      downColor: '#f23645',
      borderVisible: false,
      wickUpColor: '#089981',
      wickDownColor: '#f23645',
    });

    window.addEventListener('resize', handleResize);
  }

  function handleResize() {
    if (chart && chartContainer) {
      chart.applyOptions({
        width: chartContainer.clientWidth,
      });
    }
  }

  function handleToggleLayer(layerId: string) {
    if (chart && candleSeries) {
      layerManager.toggleLayer(layerId, getContext());
    } else {
      layerManager.toggleLayer(layerId);
    }
    layersState = layerManager.getAllLayers().map(l => ({
      id: l.id,
      name: l.name,
      visible: l.visible
    }));
  }

  function handleZoom(range: string) {
    activeRange = range;
    if (!chart || candles.length === 0) return;

    const lastCandle = candles[candles.length - 1];
    const toTime = lastCandle.time;
    let fromTime = toTime;

    const secondsInDay = 86400;
    switch (range) {
      case '1W':
        fromTime = toTime - (7 * secondsInDay);
        break;
      case '1M':
        fromTime = toTime - (30 * secondsInDay);
        break;
      case '6M':
        fromTime = toTime - (180 * secondsInDay);
        break;
      case '1Y':
        fromTime = toTime - (365 * secondsInDay);
        break;
      case 'ALL':
        chart.timeScale().fitContent();
        return;
    }

    chart.timeScale().setVisibleRange({
      from: fromTime as any,
      to: toTime as any,
    });
  }

  function updateChartData() {
    if (!candleSeries || candles.length === 0) return;

    const formattedData: CandlestickData[] = candles.map((c) => ({
      time: c.time as any,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }));

    candleSeries.setData(formattedData);

    // Render Composed Dynamic Layers
    layerManager.renderAll(getContext());

    if (activeRange === 'ALL') {
      chart?.timeScale().fitContent();
    } else {
      handleZoom(activeRange);
    }
  }

  onMount(() => {
    initChart();
    if (candles.length > 0) {
      updateChartData();
    }
    return () => {
      window.removeEventListener('resize', handleResize);
      if (chart) {
        chart.remove();
      }
    };
  });

  $effect(() => {
    if (candles.length > 0 && chart) {
      updateChartData();
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isSymbolDropdownOpen = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Unified TradingView Chart Container Card -->
<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-2.5 font-sans relative z-10">
  <!-- Unified Single Header Toolbar (TradingView Native Style) -->
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
            {activeSymbol.slice(0, 3)}/{activeSymbol.slice(3)}
          </span>

          {#if isSpecialist || (supportedSymbols && supportedSymbols.length === 1)}
            <span class="text-[9px] px-1.5 py-0.5 rounded font-extrabold bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40 font-mono">
              ⭐ GOLD SPECIALIST
            </span>
          {:else}
            {@const currentPair = activePairs.find(p => p.symbol === activeSymbol)}
            <span class="text-[9px] px-1.5 py-0.5 rounded font-bold bg-[#2a2e39] text-[#f5c344] font-mono">
              T{currentPair?.tier ?? 1} ({currentPair?.multiplier.toFixed(1) ?? '2.0'}x)
            </span>
          {/if}

          <ChevronDown class="w-3.5 h-3.5 text-[#787b86] group-hover:text-white transition-transform {isSymbolDropdownOpen ? 'rotate-180 text-[#2962ff]' : ''}" />
        </button>

        <!-- Dropdown Menu Popover -->
        {#if isSymbolDropdownOpen}
          <div
            class="fixed inset-0 z-30"
            onclick={() => isSymbolDropdownOpen = false}
            role="presentation"
          ></div>

          <div class="absolute left-0 top-full mt-1.5 w-72 bg-[#1e222d] border border-[#2a2e39] rounded-xl shadow-2xl z-40 overflow-hidden animate-in fade-in zoom-in-95 duration-150">
            <div class="p-2 border-b border-[#2a2e39] bg-[#131722]/80 flex items-center justify-between text-[11px] font-mono text-[#787b86]">
              <span class="font-bold text-[#d1d4dc]">PILIH INSTRUMEN PASAR</span>
              <span>{displayPairs.length} Pair</span>
            </div>

            <div class="p-1.5 max-h-60 overflow-y-auto space-y-1">
              {#each displayPairs as pair}
                {@const isSelected = activeSymbol === pair.symbol}
                <button
                  onclick={() => {
                    onSelectSymbol(pair.symbol);
                    isSymbolDropdownOpen = false;
                  }}
                  class="w-full flex items-center justify-between p-2 rounded-lg text-xs font-mono text-left transition-all {isSelected ? 'bg-[#2962ff] text-white shadow-sm' : 'text-[#d1d4dc] hover:bg-[#131722] hover:text-white'}"
                >
                  <div class="flex items-center gap-2">
                    <span class="font-bold">{pair.base}/{pair.quote}</span>
                    <span class="text-[9px] px-1 py-0.2 rounded font-mono {isSelected ? 'bg-white/20 text-white' : pair.tier === 1 ? 'bg-[#f5c344]/20 text-[#f5c344]' : 'bg-[#2a2e39] text-[#787b86]'}">
                      T{pair.tier} ({pair.multiplier.toFixed(1)}x)
                    </span>
                  </div>

                  {#if isSelected}
                    <Check class="w-4 h-4 text-white flex-shrink-0" />
                  {/if}
                </button>
              {/each}
            </div>

            {#if isSpecialist || (supportedSymbols && supportedSymbols.length === 1)}
              <div class="p-2 border-t border-[#2a2e39] bg-[#131722] text-[10px] text-[#f5c344] font-mono flex items-center gap-1.5">
                <Zap class="w-3 h-3 text-[#f5c344] flex-shrink-0" />
                <span>Mode Gold Specialist mengunci instrumen ke XAUUSD.</span>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Timeframe Chip -->
      <span class="text-[10px] font-semibold px-2 py-1 rounded-lg bg-[#131722] text-[#787b86] border border-[#2a2e39] font-mono">
        1H Candlestick
      </span>

      <!-- Provenance Info Badge -->
      <button
        onclick={onOpenProvenance}
        class="text-[10px] font-bold px-2.5 py-1 rounded-lg bg-[#089981]/15 hover:bg-[#089981]/25 text-[#089981] hover:text-[#26a69a] border border-[#089981]/40 hover:border-[#089981] font-mono flex items-center gap-1.5 transition-all cursor-pointer shadow-sm"
        title="Klik untuk membuka Inspektur Asal-Usul & Provenance Data Pasar"
      >
        <span>🇨🇭 DUKASCOPY ECN</span>
        <Info class="w-3 h-3 text-[#089981]" />
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
      <span class="text-[10px] text-[#787b86] font-mono px-1 font-bold flex items-center gap-1 border-r border-[#2a2e39] mr-0.5" title="Strategy-Adaptive Indicators ({applicableLayers.length} Active)">
        <Layers class="w-3.5 h-3.5 text-[#2962ff]" />
      </span>

      {#each applicableLayers as layer}
        <button
          onclick={() => handleToggleLayer(layer.id)}
          title="{layer.name} ({layer.isUniversal ? 'Universal' : 'Strategy-Specific Indicator'})"
          class="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-mono font-semibold transition-all {layer.visible ? (layer.isUniversal ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/40') : 'bg-transparent text-[#787b86] opacity-60 border border-transparent'}"
        >
          {#if layer.visible}
            <Eye class="w-3 h-3 {layer.isUniversal ? 'text-[#089981]' : 'text-[#2962ff]'}" />
          {:else}
            <EyeOff class="w-3 h-3 text-[#787b86]" />
          {/if}
          <span>{layer.shortLabel}</span>
        </button>
      {/each}
    </div>

    <!-- Right: Zoom Range Selectors & Live Sync Status -->
    <div class="flex items-center gap-2">
      {#if syncStatusMessage}
        <div class="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-[#00E676]/10 text-[#00E676] text-xs font-mono border border-[#00E676]/30 animate-pulse shadow-sm">
          <span>{syncStatusMessage}</span>
        </div>
      {/if}

      <!-- Range Zoom Selectors (TradingView Time Horizon Style) -->
      <div class="flex items-center gap-0.5 bg-[#131722] p-1 rounded-lg border border-[#2a2e39]">
        {#each ['1W', '1M', '6M', '1Y', 'ALL'] as range}
          <button
            onclick={() => handleZoom(range)}
            class="px-2 py-0.5 rounded text-[10px] font-mono font-bold transition-all {activeRange === range ? 'bg-[#2962ff] text-white shadow-sm' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
          >
            {range === 'ALL' ? 'ALL (10Y)' : range}
          </button>
        {/each}
      </div>
    </div>
  </div>

  <!-- WebGL Viewport (TradingView Dark Canvas) -->
  <div bind:this={chartContainer} class="w-full h-[520px] rounded-lg overflow-hidden border border-[#2a2e39]/60"></div>
</div>
