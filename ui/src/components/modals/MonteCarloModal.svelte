<script lang="ts">
  import type { MonteCarloReport } from '../../ports';

  interface Props {
    activeSymbol: string;
    monteCarloData: MonteCarloReport | null;
  }

  let { activeSymbol = 'XAUUSD', monteCarloData = null }: Props = $props();

  let mcRuin = $derived(monteCarloData ? monteCarloData.risk_of_ruin_pct : 0);
  let mcMedianVp = $derived(monteCarloData ? monteCarloData.median_ending_vp : 0);
  let mcWorstDd = $derived(monteCarloData ? monteCarloData.worst_case_max_dd_pct : 0);
</script>

<div class="space-y-3">
  <div class="grid grid-cols-1 sm:grid-cols-4 gap-3 mb-3">
    <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono">SIMULATION RUNS</div>
      <div class="text-xl font-bold font-mono text-[#2962ff] mt-0.5">1,000 Permutations</div>
      <div class="text-[9px] text-[#787b86] mt-0.5">Bootstrap Resampling (N={monteCarloData?.original_trades_count ?? 0})</div>
    </div>
    <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono">RISK OF RUIN (DD &gt; 3.5k VP)</div>
      <div class="text-xl font-bold font-mono {mcRuin === 0 ? 'text-[#089981]' : (mcRuin < 5 ? 'text-[#f5c344]' : 'text-[#f23645]')} mt-0.5">
        {mcRuin.toFixed(2)}%
      </div>
      <div class="text-[9px] {mcRuin === 0 ? 'text-[#089981]' : 'text-[#f5c344]'} mt-0.5">
        {mcRuin === 0 ? 'Zero Ruin Guarantee' : 'Elevated Sequence Risk'}
      </div>
    </div>
    <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono">WORST-CASE 95% DD</div>
      <div class="text-xl font-bold font-mono text-[#f23645] mt-0.5">
        -{mcWorstDd.toFixed(1)} VP
      </div>
      <div class="text-[9px] text-[#787b86] mt-0.5">5th Percentile Floor</div>
    </div>
    <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono">MEDIAN EXPECTED VP</div>
      <div class="text-xl font-bold font-mono {mcMedianVp >= 0 ? 'text-[#089981]' : 'text-[#f23645]'} mt-0.5">
        {mcMedianVp >= 0 ? '+' : ''}{mcMedianVp.toFixed(1)} VP
      </div>
      <div class="text-[9px] {mcMedianVp >= 300 ? 'text-[#089981]' : 'text-[#787b86]'} mt-0.5">
        {mcMedianVp >= 300 ? '> Target Bulanan TF' : 'Baseline Expectancy'}
      </div>
    </div>
  </div>

  <!-- Monte Carlo High-Precision SVG Equity Fan Ribbon -->
  <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39]">
    <div class="flex items-center justify-between mb-3">
      <span class="text-[11px] font-bold text-[#d1d4dc] font-mono">
        Monte Carlo Resampled Equity Paths (P5, P50, P95 &amp; Actual) • {activeSymbol}
      </span>
      <span class="text-[10px] text-[#787b86] font-mono">Confidence Level: 95%</span>
    </div>

    {#if monteCarloData && monteCarloData.equity_paths.length > 1}
      {@const pts = monteCarloData.equity_paths}
      {@const nPts = pts.length}
      {@const allVals = pts.flatMap(p => [p.p5_worst, p.p95_best, p.actual_equity, 0])}
      {@const minVal = Math.min(...allVals)}
      {@const maxVal = Math.max(...allVals)}
      {@const rangeVal = Math.max(1, maxVal - minVal)}
      
      {@const getX = (idx: number) => 20 + (idx / (nPts - 1)) * 720}
      {@const getY = (val: number) => 170 - ((val - minVal) / rangeVal) * 150}
      
      {@const p95Points = pts.map((p, i) => `${getX(i).toFixed(1)},${getY(p.p95_best).toFixed(1)}`).join(' ')}
      {@const p5PointsReversed = pts.slice().reverse().map((p, i) => `${getX(nPts - 1 - i).toFixed(1)},${getY(p.p5_worst).toFixed(1)}`).join(' ')}
      {@const fanAreaPolygon = `${p95Points} ${p5PointsReversed}`}
      {@const p50Points = pts.map((p, i) => `${getX(i).toFixed(1)},${getY(p.p50_median).toFixed(1)}`).join(' ')}
      {@const p5Points = pts.map((p, i) => `${getX(i).toFixed(1)},${getY(p.p5_worst).toFixed(1)}`).join(' ')}
      {@const actualPoints = pts.map((p, i) => `${getX(i).toFixed(1)},${getY(p.actual_equity).toFixed(1)}`).join(' ')}
      {@const zeroY = getY(0)}

      <div class="w-full bg-[#1e222d]/60 rounded-lg p-2 border border-[#2a2e39]/60">
        <svg class="w-full h-48 overflow-visible" viewBox="0 0 760 190">
          <defs>
            <linearGradient id="mcBandGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#089981" stop-opacity="0.25" />
              <stop offset="50%" stop-color="#f5c344" stop-opacity="0.15" />
              <stop offset="100%" stop-color="#f23645" stop-opacity="0.25" />
            </linearGradient>
          </defs>

          <!-- Grid Lines -->
          <line x1="20" y1="20" x2="740" y2="20" stroke="#2a2e39" stroke-width="1" stroke-dasharray="3,3" />
          <line x1="20" y1="95" x2="740" y2="95" stroke="#2a2e39" stroke-width="1" stroke-dasharray="3,3" />
          <line x1="20" y1="170" x2="740" y2="170" stroke="#2a2e39" stroke-width="1" />
          
          <!-- Zero Baseline -->
          {#if zeroY >= 20 && zeroY <= 170}
            <line x1="20" y1={zeroY} x2="740" y2={zeroY} stroke="#787b86" stroke-width="1" stroke-dasharray="4,4" opacity="0.6" />
            <text x="25" y={zeroY - 4} fill="#787b86" font-size="9" font-family="monospace">0 VP Baseline</text>
          {/if}

          <!-- 95% Confidence Band Polygon Area -->
          <polygon points={fanAreaPolygon} fill="url(#mcBandGradient)" />

          <!-- P95 (Best-Case) Line -->
          <polyline points={p95Points} fill="none" stroke="#089981" stroke-width="2" stroke-linecap="round" />

          <!-- P50 (Median) Line -->
          <polyline points={p50Points} fill="none" stroke="#f5c344" stroke-width="2" stroke-linecap="round" />

          <!-- P5 (Worst-Case) Line -->
          <polyline points={p5Points} fill="none" stroke="#f23645" stroke-width="2" stroke-linecap="round" />

          <!-- Actual Backtest Path Line -->
          <polyline points={actualPoints} fill="none" stroke="#2962ff" stroke-width="2.5" stroke-linecap="round" />
        </svg>
      </div>
    {:else}
      <div class="h-44 w-full flex items-center justify-center text-[#787b86] font-mono text-xs">
        ⏳ Memuat simulasi Monte Carlo untuk {activeSymbol}...
      </div>
    {/if}

    <div class="flex flex-wrap items-center justify-center gap-6 mt-3 text-[10px] font-mono">
      <span class="flex items-center gap-1.5 text-[#089981]">
        <span class="w-2.5 h-2.5 rounded-full bg-[#089981]"></span> 95th Percentile (Best-Case)
      </span>
      <span class="flex items-center gap-1.5 text-[#f5c344]">
        <span class="w-2.5 h-2.5 rounded-full bg-[#f5c344]"></span> 50th Percentile (Median Expected)
      </span>
      <span class="flex items-center gap-1.5 text-[#f23645]">
        <span class="w-2.5 h-2.5 rounded-full bg-[#f23645]"></span> 5th Percentile (Worst-Case Floor)
      </span>
      <span class="flex items-center gap-1.5 text-[#2962ff]">
        <span class="w-2.5 h-2.5 rounded-full bg-[#2962ff]"></span> Actual Backtest Path
      </span>
    </div>
  </div>

  <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39] text-[#787b86] leading-relaxed text-xs">
    <span class="font-bold text-[#d1d4dc]">💡 Kesimpulan Analisis Monte Carlo:</span> Dari 1.000 iterasi permutasi acak pada <strong class="text-white">{activeSymbol}</strong>, kurva P50 menghasilkan ekspektasi median <strong class="{mcMedianVp >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">{mcMedianVp >= 0 ? '+' : ''}{mcMedianVp.toFixed(1)} VP</strong>. Tingkat Risk of Ruin terukur <strong class="text-white">{mcRuin.toFixed(2)}%</strong> dengan estimasi worst-case drawdown 95% sebesar <strong class="text-[#f23645]">-{mcWorstDd.toFixed(1)} VP</strong>.
  </div>
</div>
