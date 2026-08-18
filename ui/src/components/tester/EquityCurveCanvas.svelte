<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    createChart,
    ColorType,
    BaselineSeries,
    AreaSeries,
    LineSeries,
    type IChartApi,
    type ISeriesApi,
  } from 'lightweight-charts';
  import type { EquityCurvePoint } from '../../ports/ITesterPort';
  import { TrendingUp, TrendingDown, Layers, Calendar, Award } from '@lucide/svelte';

  let { data = [] }: { data: EquityCurvePoint[] } = $props();

  type ViewMode = 'equity' | 'drawdown' | 'highwater';
  let activeMode = $state<ViewMode>('equity');

  let chartContainer: HTMLDivElement;
  let chart: IChartApi | null = null;
  let equitySeries: ISeriesApi<'Baseline'> | null = null;
  let highWaterSeries: ISeriesApi<'Line'> | null = null;
  let drawdownSeries: ISeriesApi<'Area'> | null = null;

  // Crosshair Hover State
  let hoverPoint = $state<{
    dateStr: string;
    equity: number;
    drawdown: number;
    drawdownPct: number;
    peak: number;
  } | null>(null);

  // Summary Metrics
  let latestEquity = $derived.by(() => {
    if (!data || data.length === 0) return 0;
    return Number(data[data.length - 1].equity_pips) || 0;
  });

  let maxPeakEquity = $derived.by(() => {
    if (!data || data.length === 0) return 0;
    return Math.max(...data.map((d) => Number(d.equity_pips) || 0), 0);
  });

  let maxDrawdownPips = $derived.by(() => {
    if (!data || data.length === 0) return 0;
    return Math.max(...data.map((d) => Math.abs(Number(d.drawdown_pips) || 0)), 0);
  });

  function initChart() {
    if (!chartContainer) return;

    chart = createChart(chartContainer, {
      layout: {
        background: { type: ColorType.Solid, color: '#131722' },
        textColor: '#787b86',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Trebuchet MS", Roboto, "Inter", Ubuntu, sans-serif',
        fontSize: 11,
      },
      grid: {
        vertLines: { color: 'rgba(42, 46, 57, 0.35)' },
        horzLines: { color: 'rgba(42, 46, 57, 0.35)' },
      },
      rightPriceScale: {
        borderColor: '#2a2e39',
        scaleMargins: { top: 0.1, bottom: 0.15 },
        autoScale: true,
      },
      timeScale: {
        borderColor: '#2a2e39',
        timeVisible: true,
        secondsVisible: false,
      },
      crosshair: {
        vertLine: { color: '#2962ff', width: 1, style: 2, labelBackgroundColor: '#2962ff' },
        horzLine: { color: '#2962ff', width: 1, style: 2, labelBackgroundColor: '#2962ff' },
      },
    });

    // 1. Equity Baseline Series (Green above 0, Red below 0)
    equitySeries = chart.addSeries(BaselineSeries, {
      baseValue: { type: 'price', price: 0 },
      topLineColor: '#089981',
      topFillColor1: 'rgba(8, 153, 129, 0.38)',
      topFillColor2: 'rgba(8, 153, 129, 0.02)',
      bottomLineColor: '#f23645',
      bottomFillColor1: 'rgba(242, 54, 69, 0.02)',
      bottomFillColor2: 'rgba(242, 54, 69, 0.38)',
      lineWidth: 2,
      priceFormat: {
        type: 'price',
        precision: 1,
        minMove: 0.1,
      },
      title: 'Net Equity (Pips)',
    });

    // 2. High-Water Mark Peak Step Line (Amber Dashed)
    highWaterSeries = chart.addSeries(LineSeries, {
      color: '#f59e0b',
      lineWidth: 1,
      lineStyle: 2, // Dashed
      priceFormat: {
        type: 'price',
        precision: 1,
        minMove: 0.1,
      },
      title: 'Peak High-Water Mark',
    });

    // 3. Dedicated Underwater Drawdown Area Series
    drawdownSeries = chart.addSeries(AreaSeries, {
      topColor: 'rgba(242, 54, 69, 0.02)',
      bottomColor: 'rgba(242, 54, 69, 0.45)',
      lineColor: '#f23645',
      lineWidth: 2,
      priceFormat: {
        type: 'price',
        precision: 1,
        minMove: 0.1,
      },
      title: 'Underwater Drawdown (Pips)',
    });

    // Zero Baseline on Drawdown
    drawdownSeries.createPriceLine({
      price: 0,
      color: 'rgba(120, 123, 134, 0.6)',
      lineWidth: 1,
      lineStyle: 2,
      axisLabelVisible: true,
      title: '0.0 Peak',
    });

    // Crosshair move listener for interactive HUD
    chart.subscribeCrosshairMove((param) => {
      if (!param || !param.time || !param.seriesData || !equitySeries) {
        hoverPoint = null;
        return;
      }

      const eqVal = param.seriesData.get(equitySeries) as { value?: number } | undefined;
      const peakVal = highWaterSeries
        ? (param.seriesData.get(highWaterSeries) as { value?: number } | undefined)
        : undefined;

      const timeNum = Number(param.time);
      const date = new Date(timeNum * 1000);
      const dateStr = date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });

      const eq = eqVal?.value ?? 0;
      const peak = peakVal?.value ?? Math.max(eq, 0);
      const dd = Math.max(peak - eq, 0);
      const ddPct = peak > 0 ? (dd / peak) * 100 : 0;

      hoverPoint = {
        dateStr,
        equity: eq,
        drawdown: dd,
        drawdownPct: ddPct,
        peak,
      };
    });

    applyVisibilityMode();
    updateData();
    chart.timeScale().fitContent();
  }

  function applyVisibilityMode() {
    if (!equitySeries || !drawdownSeries || !highWaterSeries) return;

    if (activeMode === 'equity') {
      equitySeries.applyOptions({ visible: true });
      highWaterSeries.applyOptions({ visible: false });
      drawdownSeries.applyOptions({ visible: false });
    } else if (activeMode === 'highwater') {
      equitySeries.applyOptions({ visible: true });
      highWaterSeries.applyOptions({ visible: true });
      drawdownSeries.applyOptions({ visible: false });
    } else if (activeMode === 'drawdown') {
      equitySeries.applyOptions({ visible: false });
      highWaterSeries.applyOptions({ visible: false });
      drawdownSeries.applyOptions({ visible: true });
    }
  }

  function updateData() {
    if (!equitySeries || !drawdownSeries || !highWaterSeries || !data || data.length === 0) return;

    const sorted = [...data].sort((a, b) => a.time - b.time);
    const seen = new Set<number>();
    const eqData: Array<{ time: any; value: number }> = [];
    const peakData: Array<{ time: any; value: number }> = [];
    const ddData: Array<{ time: any; value: number }> = [];

    let currentPeak = 0;

    for (const pt of sorted) {
      if (!seen.has(pt.time)) {
        seen.add(pt.time);
        const eq = Number(pt.equity_pips) || 0;
        if (eq > currentPeak) {
          currentPeak = eq;
        }
        const ddRaw = Math.abs(Number(pt.drawdown_pips) || 0);
        const ddNegative = -ddRaw;

        eqData.push({ time: pt.time as any, value: eq });
        peakData.push({ time: pt.time as any, value: currentPeak });
        ddData.push({ time: pt.time as any, value: ddNegative });
      }
    }

    if (eqData.length > 0) {
      equitySeries.setData(eqData);
      highWaterSeries.setData(peakData);
      drawdownSeries.setData(ddData);
      chart?.timeScale().fitContent();
    }
  }

  function setMode(mode: ViewMode) {
    activeMode = mode;
    applyVisibilityMode();
    chart?.timeScale().fitContent();
  }

  $effect(() => {
    if (data && chart) {
      updateData();
    }
  });

  onMount(() => {
    initChart();
    const handleResize = () => {
      if (chartContainer && chart) {
        chart.applyOptions({
          width: chartContainer.clientWidth,
          height: chartContainer.clientHeight,
        });
      }
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  });

  onDestroy(() => {
    if (chart) {
      chart.remove();
      chart = null;
    }
  });
</script>

<div class="relative w-full flex flex-col bg-[#131722] rounded-xl border border-[#2a2e39] overflow-hidden">
  <!-- Top Action & Interactive Metric HUD Bar -->
  <div class="flex flex-wrap items-center justify-between gap-3 px-4 py-2.5 bg-[#1e222d] border-b border-[#2a2e39]">
    <!-- Left: Dynamic Crosshair HUD or Overall KPI Badges -->
    <div class="flex items-center gap-4 text-xs font-mono">
      {#if hoverPoint}
        <div class="flex items-center gap-1.5 text-[#787b86]">
          <Calendar class="w-3.5 h-3.5 text-[#2962ff]" />
          <span class="text-[#d1d4dc] font-semibold">{hoverPoint.dateStr}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="text-[#787b86]">Equity:</span>
          <span class="font-bold {hoverPoint.equity >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
            {hoverPoint.equity >= 0 ? '+' : ''}{hoverPoint.equity.toFixed(1)} pips
          </span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="text-[#787b86]">Drawdown:</span>
          <span class="font-bold text-[#f23645]">
            -{hoverPoint.drawdown.toFixed(1)} pips ({hoverPoint.drawdownPct.toFixed(1)}%)
          </span>
        </div>
        <div class="hidden sm:flex items-center gap-1.5 text-[#787b86]">
          <Award class="w-3.5 h-3.5 text-[#f59e0b]" />
          <span>Peak:</span>
          <span class="text-[#d1d4dc] font-bold">+{hoverPoint.peak.toFixed(1)} pips</span>
        </div>
      {:else}
        <!-- Default State: Overall Performance Snapshot -->
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-1.5">
            <span class="w-2.5 h-2.5 rounded-full {latestEquity >= 0 ? 'bg-[#089981]' : 'bg-[#f23645]'}"></span>
            <span class="text-[#787b86]">Net Equity:</span>
            <span class="font-bold {latestEquity >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
              {latestEquity >= 0 ? '+' : ''}{latestEquity.toFixed(1)} pips
            </span>
          </div>
          <div class="flex items-center gap-1.5">
            <span class="w-2.5 h-2.5 rounded-full bg-[#f23645]"></span>
            <span class="text-[#787b86]">Max Drawdown:</span>
            <span class="font-bold text-[#f23645]">-{maxDrawdownPips.toFixed(1)} pips</span>
          </div>
          <div class="hidden sm:flex items-center gap-1.5">
            <span class="w-2.5 h-2.5 rounded-full bg-[#f59e0b]"></span>
            <span class="text-[#787b86]">Peak Record:</span>
            <span class="font-bold text-[#f59e0b]">+{maxPeakEquity.toFixed(1)} pips</span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Right: Mode Switcher Pills -->
    <div class="flex items-center gap-1 bg-[#131722] p-1 rounded-lg border border-[#2a2e39]">
      <button
        onclick={() => setMode('equity')}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-sans font-medium transition-all {activeMode === 'equity' ? 'bg-[#2962ff] text-white shadow-sm font-semibold' : 'text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]'}"
      >
        <TrendingUp class="w-3.5 h-3.5" /> Equity Curve
      </button>

      <button
        onclick={() => setMode('highwater')}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-sans font-medium transition-all {activeMode === 'highwater' ? 'bg-[#f59e0b] text-black shadow-sm font-semibold' : 'text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]'}"
      >
        <Layers class="w-3.5 h-3.5" /> Peak & Run-Up
      </button>

      <button
        onclick={() => setMode('drawdown')}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-sans font-medium transition-all {activeMode === 'drawdown' ? 'bg-[#f23645] text-white shadow-sm font-semibold' : 'text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]'}"
      >
        <TrendingDown class="w-3.5 h-3.5" /> Underwater DD
      </button>
    </div>
  </div>

  <!-- Chart Canvas Container -->
  <div class="relative w-full h-80 bg-[#131722]">
    <div bind:this={chartContainer} class="w-full h-full"></div>
  </div>
</div>
