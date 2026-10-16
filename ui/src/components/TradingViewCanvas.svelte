<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createChart,
    type IChartApi,
    type ISeriesApi
  } from 'lightweight-charts';
  import {
    getChartDefaultOptions,
    createSeriesForChart
  } from '../services/chartConfig';
  import ChartTypeSelector from './ChartTypeSelector.svelte';
  import { Scissors } from '@lucide/svelte';
  import type { Candle, Signal, ChartType } from '../domain/models';
  import type { SimulatedTrade, ChartLayerContext } from '../ports/layers';
  import type { IUserPreferencesPort } from '../ports/IUserPreferencesPort';
  import type { StrategyDescriptor } from '../ports';
  import { ChartLayerManager } from '../adapters/layers/ChartLayerManager';

  import { ReplayEngineService } from '../services/ReplayEngineService';
  import ReplayToolbar from './replay/ReplayToolbar.svelte';
  import GoToDateModal from './replay/GoToDateModal.svelte';
  import LayersManagerModal from './modals/LayersManagerModal.svelte';
  import ChartHeaderToolbar from './chart/ChartHeaderToolbar.svelte';
  import {
    formatSeriesDataset,
    formatSingleCandleUpdate
  } from '../services/chartSeriesFormatter';

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
    selectedSource = 'mrg_demo',
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

  let isGoToDateModalOpen = $state(false);
  let isLayersModalOpen = $state(false);
  let chartContainer: HTMLDivElement | null = $state(null);
  let chart: IChartApi | null = null;
  let candleSeries: ISeriesApi<any> | null = null;
  let activeChartType = $state<ChartType>('CANDLES');
  let currentSeriesType: ChartType = 'CANDLES';
  let activeRange = $state('1W');

  // Client TimeZone Detection
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

    chart = createChart(chartContainer, getChartDefaultOptions(chartContainer.clientWidth));

    ensureMainSeries(activeChartType);

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

    candleSeries = createSeriesForChart(chart, type, currentPrice);

    currentSeriesType = type;
  }

  function appendSingleCandle(c: Candle) {
    if (!chart || !candleSeries) return;
    const activeCandles = replayState.isActive && displayedCandles.length > 0 ? displayedCandles : candles;
    const formatted = formatSingleCandleUpdate(activeChartType, c, activeCandles);
    candleSeries.update(formatted as any);
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

    const isSingleBarUpdate =
      lastRenderedSymbol === activeSymbol &&
      lastRenderedSource === selectedSource &&
      lastRenderedChartType === activeChartType &&
      activeCandles.length === lastCandlesLength &&
      activeCandles.length > 0 &&
      !replayState.isActive;

    if (isSingleBarUpdate) {
      const lastC = activeCandles[activeCandles.length - 1];
      appendSingleCandle(lastC);
      return;
    }

    const prevLogicalRange = chart.timeScale().getVisibleLogicalRange();
    const formattedData = formatSeriesDataset(activeChartType, activeCandles, currentPrice);
    candleSeries.setData(formattedData as any);

    layerManager.renderAll(getContext());

    const isSymbolChanged = lastRenderedSymbol === '' || lastRenderedSymbol !== activeSymbol;
    const isSourceChanged = lastRenderedSource !== selectedSource;
    if ((isSymbolChanged || isSourceChanged) && !replayState.isActive) {
      if (activeRange === 'ALL' || selectedSource !== 'dukascopy') {
        chart?.timeScale().fitContent();
      } else {
        handleZoom(activeRange);
      }
    } else if (replayState.isActive && prevLogicalRange) {
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
    if (!replayState.isActive && chart) {
      const _src = selectedSource;
      const _c = candles;
      updateChartData();
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.altKey && (e.key === 'g' || e.key === 'G')) {
      e.preventDefault();
      isGoToDateModalOpen = !isGoToDateModalOpen;
      return;
    }

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

    if ((e.shiftKey && e.key === 'ArrowRight') || (replayState.isActive && e.key === 'ArrowRight')) {
      const targetTag = (e.target as HTMLElement)?.tagName;
      if (targetTag !== 'INPUT' && targetTag !== 'TEXTAREA') {
        e.preventDefault();
        replayEngine.stepForward();
        return;
      }
    }

    if ((e.shiftKey && e.key === 'ArrowLeft') || (replayState.isActive && e.key === 'ArrowLeft')) {
      const targetTag = (e.target as HTMLElement)?.tagName;
      if (targetTag !== 'INPUT' && targetTag !== 'TEXTAREA') {
        e.preventDefault();
        replayEngine.stepBackward();
        return;
      }
    }

    if (e.key === 'Escape') {
      if (isGoToDateModalOpen) {
        isGoToDateModalOpen = false;
      } else if (replayState.isSelectingCutPoint) {
        replayEngine.setSelectingCutPoint(false);
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Unified TradingView Chart Container Card -->
<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-2.5 font-sans relative z-10">
  <!-- Unified Toolbar Header -->
  <ChartHeaderToolbar
    {activeSymbol}
    {displayPairs}
    {activePairs}
    {supportedSymbols}
    {isSpecialist}
    selectedSource={selectedSource ?? 'mrg_demo'}
    {currentPrice}
    {applicableLayers}
    {syncStatusMessage}
    {activeRange}
    {userTimeZone}
    {userTimeZoneCity}
    {userTimeZoneOffset}
    {onSelectSymbol}
    {onSelectSource}
    {onSyncDelta}
    {onOpenProvenance}
    onOpenLayersModal={() => isLayersModalOpen = true}
    onToggleLayer={handleToggleLayer}
    onZoom={handleZoom}
  />

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

    <!-- Cut Line with Scissors Badge Overlay -->
    {#if replayState.isSelectingCutPoint && cutCrosshairX !== null}
      <div
        class="absolute top-0 bottom-0 pointer-events-none z-30 transition-none"
        style="left: {cutCrosshairX}px;"
      >
        <div class="w-[2px] h-full bg-[#2962ff] shadow-[0_0_14px_#2962ff]"></div>

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
