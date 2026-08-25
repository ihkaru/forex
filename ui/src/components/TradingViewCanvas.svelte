<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createChart,
    CandlestickSeries,
    LineSeries,
    AreaSeries,
    BarSeries,
    BaselineSeries,
    type IChartApi,
    type ISeriesApi,
    type CandlestickData,
    type LineData,
    type AreaData,
    type BarData,
    type BaselineData
  } from 'lightweight-charts';
  import ChartTypeSelector from './ChartTypeSelector.svelte';
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
    Check,
    Scissors,
    Clock,
    Sliders
  } from '@lucide/svelte';
  import type { Candle, Signal, ChartType } from '../domain/models';
  import type { SimulatedTrade, ChartLayerContext } from '../ports/layers';
  import type { IUserPreferencesPort } from '../ports/IUserPreferencesPort';
  import type { StrategyDescriptor } from '../ports';
  import { ChartLayerManager } from '../adapters/layers/ChartLayerManager';

  import { ReplayEngineService } from '../services/ReplayEngineService';
  import ReplayToolbar from './replay/ReplayToolbar.svelte';
  import GoToDateModal from './replay/GoToDateModal.svelte';
  import LayersManagerModal from './modals/LayersManagerModal.svelte';

  interface Props {
    activeSymbol: string;
    activePairs: Array<{ symbol: string; base: string; quote: string; tier: number; multiplier: number }>;
    supportedSymbols?: string[];
    isSpecialist?: boolean;
    activeStrategyId?: string;
    activeStrategyCategory?: string;
    strategies?: StrategyDescriptor[];
    currentPrice: number;
    candles: Candle[];
    trades: SimulatedTrade[];
    signal: Signal | null;
    syncStatusMessage?: string | null;
    selectedSource?: 'dukascopy' | 'mrg_demo' | 'mrg_real';
    preferencesPort?: IUserPreferencesPort;
    onSelectSymbol: (symbol: string) => void;
    onSelectSource?: (source: 'dukascopy' | 'mrg_demo' | 'mrg_real') => void;
    onSyncDelta?: () => void;
    onOpenProvenance?: () => void;
    onReplayChange?: (displayedCandles: Candle[], isReplayActive: boolean, latestCandle?: Candle) => void;
  }


  let {
    activeSymbol = 'XAUUSD',
    activePairs = [],
    supportedSymbols = [],
    isSpecialist = false,
    activeStrategyId = 'pola-n-v3',
    activeStrategyCategory = 'GOLD_SPECIALIST',
    strategies = [],
    currentPrice = 0.85412,
    candles = [],
    trades = [],
    signal = null,
    syncStatusMessage = null,
    selectedSource = 'dukascopy',
    preferencesPort = undefined,
    onSelectSymbol,
    onSelectSource,
    onSyncDelta,
    onOpenProvenance,
    onReplayChange
  }: Props = $props();




  let displayPairs = $derived(
    supportedSymbols && supportedSymbols.length > 0
      ? activePairs.filter((p) => supportedSymbols.includes(p.symbol))
      : activePairs
  );

  let isSymbolDropdownOpen = $state(false);
  let isGoToDateModalOpen = $state(false);
  let isLayersModalOpen = $state(false);
  let chartContainer: HTMLDivElement | null = $state(null);
  let chart: IChartApi | null = null;
  let candleSeries: ISeriesApi<any> | null = null;
  let activeChartType = $state<ChartType>('CANDLES');
  let currentSeriesType: ChartType = 'CANDLES';
  let activeRange = $state('1W');

  // Client TimeZone Detection (TradingView Native Standard)
  const userTimeZone = typeof Intl !== 'undefined' ? Intl.DateTimeFormat().resolvedOptions().timeZone : 'UTC';
  const userTimeZoneOffset = (() => {
    try {
      const d = new Date();
      const offsetMinutes = -d.getTimezoneOffset();
      const sign = offsetMinutes >= 0 ? '+' : '-';
      const hours = Math.floor(Math.abs(offsetMinutes) / 60);
      return `UTC${sign}${hours}`;
    } catch {
      return 'UTC';
    }
  })();
  const userTimeZoneCity = userTimeZone.split('/').pop()?.replace(/_/g, ' ') || userTimeZone;

  // Interactive Live Cut Line Tracking
  let cutCrosshairX = $state<number | null>(null);
  let cutCrosshairDateStr = $state('');
  let cutCrosshairPrice = $state<number | null>(null);

  // Pure Composition Root: Chart Layer Manager & Replay Engine Service
  const layerManager = new ChartLayerManager();
  const replayEngine = new ReplayEngineService();

  let lastRenderedSource = '';
  let replayState = $state(replayEngine.getState());
  let displayedCandles: Candle[] = [];



  replayEngine.subscribe((st, sliced, latestCandle, isStepForward) => {
    const wasSelecting = replayState.isSelectingCutPoint;
    const wasActive = replayState.isActive;
    replayState = st;

    if (!st.isSelectingCutPoint) {
      cutCrosshairX = null;
    }

    // If only toggling cut selection mode (not starting replay or stepping), do NOT reload chart data or reset viewport!
    if (!st.isActive && !wasActive && st.isSelectingCutPoint !== wasSelecting) {
      return;
    }

    if (isStepForward && latestCandle) {
      displayedCandles.push(latestCandle);
      appendSingleCandle(latestCandle);
      if (onReplayChange) {
        onReplayChange(displayedCandles, st.isActive, latestCandle);
      }
    } else {
      displayedCandles = sliced;
      updateChartData();
      if (onReplayChange) {
        onReplayChange(sliced, st.isActive);
      }
    }
  });


  $effect(() => {
    if (candles && candles.length > 0) {
      replayEngine.loadDataset(candles);
    }
  });


  function syncPreferences() {
    const prefs = preferencesPort?.loadPreferences();
    if (prefs?.activeZoomRange) {
      activeRange = prefs.activeZoomRange;
    }
    if (prefs?.layerVisibility) {
      for (const [layerId, isVisible] of Object.entries(prefs.layerVisibility)) {
        const layer = layerManager.getLayer(layerId);
        if (layer && layer.visible !== isVisible) {
          layerManager.toggleLayer(layerId);
        }
      }
    }
  }

  let layersState = $state(layerManager.getAllLayers().map(l => ({
    id: l.id,
    name: l.name,
    visible: l.visible
  })));

  let applicableLayers = $derived(
    layerManager.getApplicableLayers(activeStrategyId, activeStrategyCategory)
  );

  function getContext(): ChartLayerContext {
    const activeCandles = replayState.isActive && displayedCandles.length > 0 ? displayedCandles : candles;
    return {
      chart: chart!,
      candleSeries: candleSeries!,
      candles: activeCandles,
      trades,
      signal,
      activeSymbol,
      activeStrategyId,
      activeStrategyCategory,
    };
  }

  let currentLayerContext = $derived.by<ChartLayerContext | null>(() => {
    if (!chart || !candleSeries) return null;
    return getContext();
  });

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
      localization: {
        timeFormatter: (timestamp: number) => {
          const d = new Date(timestamp * 1000);
          return new Intl.DateTimeFormat(undefined, {
            day: '2-digit',
            month: 'short',
            year: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
            hour12: false,
          }).format(d);
        },
      },
      timeScale: {
        borderColor: '#2a2e39',
        timeVisible: true,
        secondsVisible: false,
      },
      rightPriceScale: {
        borderColor: '#2a2e39',
        autoScale: true,
        scaleMargins: {
          top: 0.12,
          bottom: 0.12,
        },
      },
    });

    ensureMainSeries(activeChartType);

    // TradingView Replay Live Blue Cut Line Mouse Tracking
    chart.subscribeCrosshairMove((param) => {
      if (replayState.isSelectingCutPoint && param.point) {
        cutCrosshairX = param.point.x;
        if (param.time) {
          const t = typeof param.time === 'number' ? param.time : (param.time as any).timestamp;
          if (t) {
            cutCrosshairDateStr = new Intl.DateTimeFormat(undefined, {
              weekday: 'short',
              day: '2-digit',
              month: 'short',
              year: 'numeric',
              hour: '2-digit',
              minute: '2-digit',
              hour12: false,
              timeZoneName: 'short',
            }).format(new Date(t * 1000));
          }
        }
        if (param.seriesData && candleSeries) {
          const cData = param.seriesData.get(candleSeries) as any;
          if (cData && typeof cData.close === 'number') {
            cutCrosshairPrice = cData.close;
          }
        }
      } else {
        cutCrosshairX = null;
      }
    });

    // TradingView Replay Click-to-Cut Subscription
    chart.subscribeClick((param) => {
      if (!param.time) return;
      if (replayState.isSelectingCutPoint) {
        const timeSec = typeof param.time === 'number' ? param.time : (param.time as any).timestamp || 0;
        if (timeSec) {
          replayEngine.startReplayAtTime(timeSec);
        }
      }
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

    if (preferencesPort) {
      const visibilityMap: Record<string, boolean> = {};
      for (const l of layerManager.getAllLayers()) {
        visibilityMap[l.id] = l.visible;
      }
      preferencesPort.savePreferences({ layerVisibility: visibilityMap });
    }
  }

  function handleZoom(range: string) {
    activeRange = range;
    preferencesPort?.savePreferences({ activeZoomRange: range });
    const activeCandles = replayState.isActive && displayedCandles.length > 0 ? displayedCandles : candles;
    if (!chart || activeCandles.length === 0) return;

    const lastCandle = activeCandles[activeCandles.length - 1];
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

  let lastRenderedSymbol = '';
  let lastRenderedChartType: ChartType = 'CANDLES';
  let lastCandlesLength = 0;
  let lastReplayActive = false;

  function ensureMainSeries(type: ChartType) {
    if (!chart) return;
    const isCandleVariant = (t: ChartType) => t === 'CANDLES' || t === 'VOLUME_CANDLES' || t === 'HEIKIN_ASHI';

    if (candleSeries && isCandleVariant(type) && isCandleVariant(currentSeriesType)) {
      currentSeriesType = type;
      return;
    }

    if (candleSeries) {
      try {
        chart.removeSeries(candleSeries);
      } catch (e) {
        console.warn('Could not cleanly remove existing candle series:', e);
      }
      candleSeries = null;
    }

    if (type === 'LINE') {
      candleSeries = chart.addSeries(LineSeries, {
        color: '#2962ff',
        lineWidth: 2,
        priceLineVisible: false,
      });
    } else if (type === 'AREA') {
      candleSeries = chart.addSeries(AreaSeries, {
        topColor: 'rgba(41, 98, 255, 0.4)',
        bottomColor: 'rgba(41, 98, 255, 0.02)',
        lineColor: '#2962ff',
        lineWidth: 2,
        priceLineVisible: false,
      });
    } else if (type === 'BARS') {
      candleSeries = chart.addSeries(BarSeries, {
        upColor: '#089981',
        downColor: '#f23645',
        priceLineVisible: false,
      });
    } else if (type === 'BASELINE') {
      candleSeries = chart.addSeries(BaselineSeries, {
        baseValue: { type: 'price', price: currentPrice },
        topFillColor1: 'rgba(8, 153, 129, 0.24)',
        topFillColor2: 'rgba(8, 153, 129, 0.04)',
        bottomFillColor1: 'rgba(242, 54, 69, 0.04)',
        bottomFillColor2: 'rgba(242, 54, 69, 0.24)',
        topLineColor: '#089981',
        bottomLineColor: '#f23645',
        priceLineVisible: false,
      });
    } else {
      // CANDLES, VOLUME_CANDLES, HEIKIN_ASHI
      candleSeries = chart.addSeries(CandlestickSeries, {
        upColor: '#089981',
        downColor: '#f23645',
        borderVisible: false,
        wickUpColor: '#089981',
        wickDownColor: '#f23645',
      });
    }

    currentSeriesType = type;
  }

  function appendSingleCandle(c: Candle) {
    if (!chart || !candleSeries) return;

    if (activeChartType === 'LINE' || activeChartType === 'AREA' || activeChartType === 'BASELINE') {
      candleSeries.update({
        time: c.time as any,
        value: c.close,
      } as any);
    } else if (activeChartType === 'VOLUME_CANDLES') {
      const activeCandles = displayedCandles;
      const lastIdx = activeCandles.length - 1;
      const start = Math.max(0, lastIdx - 19);
      let sum = 0;
      for (let j = start; j <= lastIdx; j++) {
        sum += activeCandles[j]?.volume || 1.0;
      }
      const avgVol = sum / (lastIdx - start + 1);
      const vol = c.volume || 1.0;
      const isHighVol = vol > avgVol * 1.5;
      const isUp = c.close >= c.open;
      const color = isHighVol ? (isUp ? '#00f2fe' : '#ff0055') : (isUp ? '#089981' : '#f23645');
      candleSeries.update({
        time: c.time as any,
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
        color,
        wickColor: color,
      } as any);
    } else if (activeChartType === 'HEIKIN_ASHI') {
      const activeCandles = displayedCandles;
      const lastIdx = activeCandles.length - 1;
      const prev = lastIdx > 0 ? activeCandles[lastIdx - 1] : c;
      const prevClose = (prev.open + prev.high + prev.low + prev.close) / 4;
      const prevOpen = (prev.open + prev.close) / 2;
      const close = (c.open + c.high + c.low + c.close) / 4;
      const open = (prevOpen + prevClose) / 2;
      const high = Math.max(c.high, open, close);
      const low = Math.min(c.low, open, close);
      candleSeries.update({
        time: c.time as any,
        open,
        high,
        low,
        close,
      } as any);
    } else {
      candleSeries.update({
        time: c.time as any,
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
      } as any);
    }

    layerManager.updateAll(getContext(), c);
  }

  function updateChartData() {
    if (!chart) return;
    ensureMainSeries(activeChartType);
    if (!candleSeries) return;

    const activeCandles = replayState.isActive && displayedCandles.length > 0 ? displayedCandles : candles;
    if (activeCandles.length === 0) {
      if (candleSeries) {
        candleSeries.setData([]);
      }
      layerManager.clearAll();
      return;
    }

    // Capture the current visible logical range before dataset modification
    const prevLogicalRange = chart.timeScale().getVisibleLogicalRange();

    if (activeChartType === 'LINE' || activeChartType === 'AREA' || activeChartType === 'BASELINE') {
      const lineData: LineData[] = activeCandles.map((c) => ({
        time: c.time as any,
        value: c.close,
      }));
      candleSeries.setData(lineData as any);
    } else if (activeChartType === 'VOLUME_CANDLES') {
      const len = activeCandles.length;
      const volData: any[] = [];
      for (let i = 0; i < len; i++) {
        const c = activeCandles[i];
        const start = Math.max(0, i - 19);
        let sum = 0;
        for (let j = start; j <= i; j++) {
          sum += activeCandles[j].volume || 1.0;
        }
        const avgVol = sum / (i - start + 1);
        const vol = c.volume || 1.0;
        const isHighVol = vol > avgVol * 1.5;
        const isUp = c.close >= c.open;

        const color = isHighVol
          ? (isUp ? '#00f2fe' : '#ff0055')
          : (isUp ? '#089981' : '#f23645');

        volData.push({
          time: c.time as any,
          open: c.open,
          high: c.high,
          low: c.low,
          close: c.close,
          color,
          wickColor: color,
        });
      }
      candleSeries.setData(volData as any);
    } else if (activeChartType === 'HEIKIN_ASHI') {
      const haData: CandlestickData[] = [];
      for (let i = 0; i < activeCandles.length; i++) {
        const c = activeCandles[i];
        const close = (c.open + c.high + c.low + c.close) / 4;
        const open = i === 0 ? (c.open + c.close) / 2 : (haData[i - 1].open + haData[i - 1].close) / 2;
        const high = Math.max(c.high, open, close);
        const low = Math.min(c.low, open, close);
        haData.push({
          time: c.time as any,
          open,
          high,
          low,
          close,
        });
      }
      candleSeries.setData(haData as any);
    } else {
      const standardData: CandlestickData[] = activeCandles.map((c) => ({
        time: c.time as any,
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
      }));
      candleSeries.setData(standardData as any);
    }

    layerManager.renderAll(getContext());

    // Only apply initial auto-zoom on symbol/source change or initial load (NEVER during user pan/scroll or bar replay cut)
    const isSymbolChanged = lastRenderedSymbol === '' || lastRenderedSymbol !== activeSymbol;
    const isSourceChanged = lastRenderedSource !== selectedSource;
    if ((isSymbolChanged || isSourceChanged) && !replayState.isActive) {
      if (activeRange === 'ALL' || selectedSource !== 'dukascopy') {
        chart?.timeScale().fitContent();
      } else {
        handleZoom(activeRange);
      }
    } else if (replayState.isActive && prevLogicalRange) {
      // TradingView Seamless Replay: Lock and preserve exact visible viewport so the cut bar stays at exact screen position!
      try {
        chart.timeScale().setVisibleLogicalRange(prevLogicalRange);
      } catch (e) {}
    }

    lastRenderedSymbol = activeSymbol;
    lastRenderedSource = selectedSource;
    lastRenderedChartType = activeChartType;
    lastCandlesLength = activeCandles.length;
    lastReplayActive = replayState.isActive;
  }



  onMount(() => {
    syncPreferences();
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
    // Re-render when activeSymbol, selectedSource, activeChartType, or dataset updates
    if (!replayState.isActive && chart) {
      const _src = selectedSource;
      const _c = candles;
      updateChartData();
    }
  });



  function handleKeydown(e: KeyboardEvent) {
    // Alt + G: Jump to Date
    if (e.altKey && (e.key === 'g' || e.key === 'G')) {
      e.preventDefault();
      isGoToDateModalOpen = !isGoToDateModalOpen;
      return;
    }

    // Space: Play / Pause toggle
    if (e.key === ' ' && replayState.isActive && !isGoToDateModalOpen) {
      const targetTag = (e.target as HTMLElement)?.tagName;
      if (targetTag !== 'INPUT' && targetTag !== 'TEXTAREA') {
        e.preventDefault();
        if (replayState.isPlaying) {
          replayEngine.pause();
        } else {
          replayEngine.play();
        }
      }
      return;
    }

    // Shift + ArrowRight or ArrowRight (when in replay): Step Forward
    if ((e.shiftKey && e.key === 'ArrowRight') || (replayState.isActive && e.key === 'ArrowRight')) {
      const targetTag = (e.target as HTMLElement)?.tagName;
      if (targetTag !== 'INPUT' && targetTag !== 'TEXTAREA') {
        e.preventDefault();
        replayEngine.stepForward();
        return;
      }
    }

    // Shift + ArrowLeft or ArrowLeft (when in replay): Step Backward
    if ((e.shiftKey && e.key === 'ArrowLeft') || (replayState.isActive && e.key === 'ArrowLeft')) {
      const targetTag = (e.target as HTMLElement)?.tagName;
      if (targetTag !== 'INPUT' && targetTag !== 'TEXTAREA') {
        e.preventDefault();
        replayEngine.stepBackward();
        return;
      }
    }

    // Escape
    if (e.key === 'Escape') {
      if (isGoToDateModalOpen) {
        isGoToDateModalOpen = false;
      } else if (replayState.isSelectingCutPoint) {
        replayEngine.setSelectingCutPoint(false);
      } else {
        isSymbolDropdownOpen = false;
      }
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
        1H
      </span>

      <!-- TradingView Chart Type Selector Dropdown -->
      <ChartTypeSelector
        activeType={activeChartType}
        onSelectType={(type) => {
          activeChartType = type;
          updateChartData();
        }}
      />

      <!-- TradingView Bar Replay Trigger Button -->
      <button
        onclick={() => {
          if (replayState.isActive) {
            replayEngine.stopReplay();
          } else {
            replayEngine.setSelectingCutPoint(!replayState.isSelectingCutPoint);
          }
        }}
        class="text-[10px] font-bold px-2.5 py-1 rounded-lg flex items-center gap-1.5 transition-all cursor-pointer shadow-sm {replayState.isActive ? 'bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40 hover:bg-[#f23645]/30' : replayState.isSelectingCutPoint ? 'bg-[#2962ff] text-white animate-pulse' : 'bg-[#131722] hover:bg-[#2a2e39] text-[#787b86] hover:text-white border border-[#2a2e39]'}"
        title="Buka Fitur Bar Replay (Simulasi Bar-by-Bar Tanpa Lookahead)"
      >
        <Scissors class="w-3 h-3 {replayState.isSelectingCutPoint ? 'animate-bounce' : ''}" />
        <span>{replayState.isActive ? 'REPLAYING' : 'BAR REPLAY'}</span>
      </button>

      <!-- Interactive Provenance Source Switcher Pill -->
      <div class="flex items-center rounded-lg bg-[#131722] border border-[#2a2e39] p-0.5 font-mono text-[10px]">
        <button
          onclick={() => onSelectSource?.('dukascopy')}
          class="px-2 py-0.5 rounded font-bold transition-all {selectedSource === 'dukascopy' ? 'bg-[#089981] text-white shadow-sm' : 'text-[#787b86] hover:text-white'}"
          title="Gunakan Data 10 Tahun Dukascopy ECN (Backtest / Research)"
        >
          🇨🇭 DUKASCOPY (10Y)
        </button>
        <button
          onclick={() => onSelectSource?.('mrg_demo')}
          class="px-2 py-0.5 rounded font-bold transition-all {selectedSource === 'mrg_demo' ? 'bg-[#2962ff] text-white shadow-sm' : 'text-[#787b86] hover:text-white'}"
          title="Gunakan Data Live MT4 Broker MRG Demo (Simulasi / Staging Feed)"
        >
          🧪 MRG DEMO
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
        onclick={() => isLayersModalOpen = true}
        class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-mono font-bold text-[#787b86] hover:text-white hover:bg-[#2a2e39] border-r border-[#2a2e39] mr-0.5 transition-all"
        title="Buka Object Tree & Pengaturan Layer Lengkap"
      >
        <Layers class="w-3.5 h-3.5 text-[#2962ff]" />
        <Sliders class="w-3 h-3 text-[#787b86]" />
      </button>

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

    <!-- Right: Timezone/Volume Badge, Zoom Range Selectors & Live Sync Status -->
    <div class="flex items-center gap-2">
      <!-- Timezone & Liquidity Badge (Cleanly docked in toolbar, unobstructed chart) -->
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

  <!-- Floating Replay Toolbar Overlay when selecting cut point or active replay -->
  {#if replayState.isSelectingCutPoint || replayState.isActive}
    <div class="relative z-30 mb-2 animate-in fade-in slide-in-from-top-2 duration-150">
      <ReplayToolbar
        {replayEngine}
        {replayState}
        onOpenGoToDate={() => isGoToDateModalOpen = true}
      />
    </div>
  {/if}

  <!-- WebGL Viewport Container with Cut Line Overlay -->
  <div class="relative w-full h-[520px] rounded-lg overflow-hidden border border-[#2a2e39]/60 {replayState.isSelectingCutPoint ? 'cursor-crosshair ring-2 ring-[#2962ff]/50' : ''}">
    <div bind:this={chartContainer} class="w-full h-full"></div>

    <!-- TradingView Authentic Blue Vertical Cut Line with Scissors Badge Overlay -->
    {#if replayState.isSelectingCutPoint && cutCrosshairX !== null}
      <div
        class="absolute top-0 bottom-0 pointer-events-none z-30 transition-none"
        style="left: {cutCrosshairX}px;"
      >
        <!-- Blue Vertical Glowing Line -->
        <div class="w-[2px] h-full bg-[#2962ff] shadow-[0_0_14px_#2962ff]"></div>

        <!-- Glowing Scissors + Date Pill at Top -->
        <div class="absolute top-3 -left-36 w-72 flex flex-col items-center gap-1 bg-[#131722]/95 border border-[#2962ff] text-white px-2.5 py-1.5 rounded-xl shadow-2xl backdrop-blur-md animate-in fade-in zoom-in-95 duration-100 font-mono text-center">
          <div class="flex items-center gap-1.5 text-xs font-black text-[#2962ff]">
            <Scissors class="w-3.5 h-3.5 animate-bounce" />
            <span>KLIK UNTUK MEMOTONG</span>
          </div>
          <div class="text-[10px] text-[#d1d4dc] font-extrabold">
            {cutCrosshairDateStr || 'Pilih Bar'}
          </div>
          {#if cutCrosshairPrice !== null}
            <div class="text-[9px] text-[#089981] font-bold">
              Harga: {cutCrosshairPrice.toFixed(5)}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>


  <!-- Go to Date Modal Popup (Alt + G) -->
  <GoToDateModal
    isOpen={isGoToDateModalOpen}
    {candles}
    {replayEngine}
    onClose={() => isGoToDateModalOpen = false}
  />

  <!-- Object Tree & Layers Manager Modal -->
  <LayersManagerModal
    isOpen={isLayersModalOpen}
    layers={applicableLayers}
    {strategies}
    {activeStrategyId}
    layerContext={currentLayerContext}
    onToggleLayer={handleToggleLayer}
    onClose={() => isLayersModalOpen = false}
  />
</div>

