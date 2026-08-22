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
    Stethoscope,
    ShieldAlert,
    Cpu,
    RefreshCw,
    Info
  } from '@lucide/svelte';
  import type { Candle, Signal } from '../domain/models';
  import type { SimulatedTrade } from '../ports/layers';
  import { ChartLayerManager } from '../adapters/layers/ChartLayerManager';

  interface Props {
    activeSymbol: string;
    activePairs: Array<{ symbol: string; base: string; quote: string; tier: number; multiplier: number }>;
    currentPrice: number;
    candles: Candle[];
    trades: SimulatedTrade[];
    signal: Signal | null;
    syncStatusMessage?: string | null;
    onSelectSymbol: (symbol: string) => void;
    onScanSignal: () => void;
    onOpenEda: () => void;
    onSyncDelta?: () => void;
    onOpenProvenance?: () => void;
  }

  let {
    activeSymbol = 'EURGBP',
    activePairs = [],
    currentPrice = 0.85412,
    candles = [],
    trades = [],
    signal = null,
    syncStatusMessage = null,
    onSelectSymbol,
    onScanSignal,
    onOpenEda,
    onSyncDelta,
    onOpenProvenance
  }: Props = $props();

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

  function getContext() {
    return {
      chart: chart!,
      candleSeries: candleSeries!,
      candles,
      trades,
      signal,
      activeSymbol
    };
  }

  function initChart() {
    if (!chartContainer) return;

    chart = createChart(chartContainer, {
      width: chartContainer.clientWidth,
      height: 480,
      layout: {
        background: { color: '#131722' },
        textColor: '#787b86',
        fontFamily: "-apple-system, BlinkMacSystemFont, 'Trebuchet MS', Roboto, Ubuntu, sans-serif",
        fontSize: 11,
      },
      grid: {
        vertLines: { color: 'rgba(42, 46, 57, 0.4)' },
        horzLines: { color: 'rgba(42, 46, 57, 0.4)' },
      },
      crosshair: {
        mode: 1,
        vertLine: { color: '#2962ff', width: 1, style: 2 },
        horzLine: { color: '#2962ff', width: 1, style: 2 },
      },
      rightPriceScale: {
        borderColor: '#2a2e39',
        scaleMargins: { top: 0.15, bottom: 0.2 },
        autoScale: true,
      },
      timeScale: {
        borderColor: '#2a2e39',
        timeVisible: true,
        secondsVisible: false,
      },
    });

    candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#089981',
      downColor: '#f23645',
      borderVisible: true,
      borderUpColor: '#089981',
      borderDownColor: '#f23645',
      wickUpColor: '#089981',
      wickDownColor: '#f23645',
      priceFormat: {
        type: 'price',
        precision: 5,
        minMove: 0.00001,
      },
    });

    const resizeObserver = new ResizeObserver((entries) => {
      if (entries.length > 0 && chart && chartContainer) {
        chart.applyOptions({ width: chartContainer.clientWidth });
      }
    });
    resizeObserver.observe(chartContainer);
  }

  function updateChartData() {
    if (!candleSeries || !chart || candles.length === 0) return;

    const chartData: CandlestickData[] = candles.map((c) => ({
      time: c.time as any,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }));

    candleSeries.setData(chartData);

    // Render all composed layers
    const ctx = getContext();
    layerManager.renderAll(ctx);

    // Initial View: Latest 150 bars for clear visual inspection
    const total = candles.length;
    if (total > 150) {
      chart.timeScale().setVisibleLogicalRange({
        from: total - 150,
        to: total,
      });
    } else {
      chart.timeScale().fitContent();
    }
  }

  function handleToggleLayer(layerId: string) {
    if (!chart || !candleSeries) return;
    const ctx = getContext();
    layerManager.toggleLayer(layerId, ctx);
    layersState = layerManager.getAllLayers().map(l => ({
      id: l.id,
      name: l.name,
      visible: l.visible
    }));
  }

  function handleZoom(range: string) {
    if (!chart || candles.length === 0) return;
    activeRange = range;
    const total = candles.length;

    if (range === 'ALL') {
      chart.timeScale().fitContent();
    } else {
      const bars = range === '1W' ? 150 : range === '1M' ? 720 : range === '6M' ? 4320 : 8760;
      chart.timeScale().setVisibleLogicalRange({
        from: Math.max(0, total - bars),
        to: total,
      });
    }
  }

  onMount(() => {
    initChart();
    if (candles.length > 0) {
      updateChartData();
    }
  });

  $effect(() => {
    if (candles.length > 0 && chart) {
      updateChartData();
    }
  });
</script>

<div class="flex flex-col gap-3 font-sans">
  <!-- Pair Selector & Global Actions Toolbar (TradingView Bar) -->
  <div class="flex flex-wrap items-center justify-between gap-3 bg-[#1e222d] p-2 rounded-xl border border-[#2a2e39]">
    <div class="flex flex-wrap items-center gap-1.5">
      {#each activePairs as pair}
        <button
          onclick={() => onSelectSymbol(pair.symbol)}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-mono font-bold transition-all {activeSymbol === pair.symbol ? 'bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/50 shadow-sm' : 'bg-[#131722] text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39] border border-transparent'}"
        >
          <span>{pair.base}/{pair.quote}</span>
          <span class="text-[9px] px-1 py-0.2 rounded font-mono {pair.tier === 1 ? 'bg-[#f5c344]/20 text-[#f5c344]' : 'bg-[#2a2e39] text-[#787b86]'}">
            T{pair.tier} ({pair.multiplier.toFixed(1)}x)
          </span>
        </button>
      {/each}
    </div>

    <!-- Action Buttons -->
    <div class="flex items-center gap-2">
      {#if syncStatusMessage}
        <div class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#00E676]/10 text-[#00E676] text-xs font-mono border border-[#00E676]/30 animate-pulse shadow-sm">
          <span>{syncStatusMessage}</span>
        </div>
      {/if}

      <button
        onclick={onSyncDelta}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#131722] hover:bg-[#2a2e39] text-[#00E676] hover:text-[#69f0ae] text-xs font-semibold border border-[#2a2e39] hover:border-[#00E676]/50 transition-all shadow-sm"
        title="Trigger Continuous Delta Sync (High-Watermark Ingestion)"
      >
        <RefreshCw class="w-3.5 h-3.5" /> Sync Delta
      </button>

      <button
        onclick={onOpenEda}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#131722] hover:bg-[#2a2e39] text-[#d1d4dc] text-xs font-semibold border border-[#2a2e39] transition-all shadow-sm"
      >
        <Stethoscope class="w-3.5 h-3.5 text-[#2962ff]" /> Audit EDA Data
      </button>

      <button
        onclick={onScanSignal}
        class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg bg-[#2962ff] hover:bg-[#1e53e5] text-white text-xs font-bold font-mono transition-all shadow-md shadow-[#2962ff]/20"
      >
        <Zap class="w-3.5 h-3.5" /> Scan Sinyal Pola N
      </button>
    </div>
  </div>

  <!-- Chart Container Card (TradingView Theme) -->
  <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-4 shadow-md flex flex-col gap-3">
    <!-- Chart Header with Price & Multi-Year Range Buttons -->
    <div class="flex flex-wrap items-center justify-between gap-3 pb-2 border-b border-[#2a2e39]">
      <div class="flex items-center gap-3">
        <span class="text-base font-black font-mono text-white tracking-wide">
          {activeSymbol.slice(0, 3)}/{activeSymbol.slice(3)}
        </span>
        <span class="text-[10px] font-semibold px-2 py-0.5 rounded bg-[#131722] text-[#787b86] border border-[#2a2e39] font-mono">
          H1 Candlestick
        </span>
        <button
          onclick={onOpenProvenance}
          class="text-[10px] font-bold px-2 py-0.5 rounded bg-[#089981]/20 hover:bg-[#089981]/30 text-[#089981] hover:text-[#26a69a] border border-[#089981]/40 hover:border-[#089981] font-mono flex items-center gap-1 transition-all cursor-pointer shadow-sm"
          title="Klik untuk membuka Inspektur Asal-Usul & Provenance Data Pasar"
        >
          <span>🇨🇭 DUKASCOPY ECN (SWISS BANK)</span>
          <Info class="w-3 h-3 text-[#089981]" />
        </button>
        <span class="text-base font-black font-mono text-[#089981]">
          {currentPrice.toFixed(5)}
        </span>
        <span class="text-xs text-[#787b86] font-mono">Spread: 0.8 pips</span>

        <!-- Range Zoom Selectors -->
        <div class="flex items-center gap-1 bg-[#131722] p-1 rounded-lg border border-[#2a2e39] ml-2">
          {#each ['1W', '1M', '6M', '1Y', 'ALL'] as range}
            <button
              onclick={() => handleZoom(range)}
              class="px-2 py-0.5 rounded text-[10px] font-mono font-bold transition-all {activeRange === range ? 'bg-[#2962ff] text-white' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
            >
              {range === 'ALL' ? 'ALL (10Y)' : range}
            </button>
          {/each}
        </div>
      </div>

      <!-- TradingView Layer Toggles (Eye Controls) -->
      <div class="flex items-center gap-1.5 bg-[#131722] p-1.5 rounded-xl border border-[#2a2e39]">
        <div class="flex items-center gap-1 text-[11px] font-mono text-[#787b86] font-bold px-1 mr-1 border-r border-[#2a2e39]">
          <Layers class="w-3.5 h-3.5 text-[#2962ff]" /> Layers:
        </div>

        <!-- Layer 1: Backtest Trades -->
        <button
          onclick={() => handleToggleLayer('backtest-trades')}
          title="Toggle Backtest Trades Markers (Win/Loss Arrows)"
          class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-[11px] font-mono font-semibold transition-all {layersState.find(l => l.id === 'backtest-trades')?.visible ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#1e222d] text-[#787b86] opacity-60 border border-transparent'}"
        >
          {#if layersState.find(l => l.id === 'backtest-trades')?.visible}
            <Eye class="w-3 h-3 text-[#089981]" />
          {:else}
            <EyeOff class="w-3 h-3 text-[#787b86]" />
          {/if}
          <span>Trades</span>
        </button>

        <!-- Layer 2: Pola N Swings -->
        <button
          onclick={() => handleToggleLayer('pola-n-swings')}
          title="Toggle Fractal Swings (H1/L1/H2/L2)"
          class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-[11px] font-mono font-semibold transition-all {layersState.find(l => l.id === 'pola-n-swings')?.visible ? 'bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40' : 'bg-[#1e222d] text-[#787b86] opacity-60 border border-transparent'}"
        >
          {#if layersState.find(l => l.id === 'pola-n-swings')?.visible}
            <Eye class="w-3 h-3 text-[#f5c344]" />
          {:else}
            <EyeOff class="w-3 h-3 text-[#787b86]" />
          {/if}
          <span>Swings</span>
        </button>

        <!-- Layer 3: Dual EMA -->
        <button
          onclick={() => handleToggleLayer('dual-ema')}
          title="Toggle EMA 20 & EMA 50"
          class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-[11px] font-mono font-semibold transition-all {layersState.find(l => l.id === 'dual-ema')?.visible ? 'bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/40' : 'bg-[#1e222d] text-[#787b86] opacity-60 border border-transparent'}"
        >
          {#if layersState.find(l => l.id === 'dual-ema')?.visible}
            <Eye class="w-3 h-3 text-[#2962ff]" />
          {:else}
            <EyeOff class="w-3 h-3 text-[#787b86]" />
          {/if}
          <span>EMA (20/50)</span>
        </button>

        <!-- Layer 4: Signal Overlay -->
        <button
          onclick={() => handleToggleLayer('signal-overlay')}
          title="Toggle Signal R:R Bounds (Entry/SL/TP)"
          class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-[11px] font-mono font-semibold transition-all {layersState.find(l => l.id === 'signal-overlay')?.visible ? 'bg-[#f5c344]/20 text-[#f5c344] border border-[#f5c344]/40' : 'bg-[#1e222d] text-[#787b86] opacity-60 border border-transparent'}"
        >
          {#if layersState.find(l => l.id === 'signal-overlay')?.visible}
            <Eye class="w-3 h-3 text-[#f5c344]" />
          {:else}
            <EyeOff class="w-3 h-3 text-[#787b86]" />
          {/if}
          <span>Signal R:R</span>
        </button>
      </div>
    </div>

    <!-- WebGL Viewport (TradingView Dark Canvas) -->
    <div bind:this={chartContainer} class="w-full h-[480px] rounded-lg overflow-hidden border border-[#2a2e39]/60"></div>
  </div>
</div>
