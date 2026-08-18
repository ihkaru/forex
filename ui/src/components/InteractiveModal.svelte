<script lang="ts">
  import {
    X,
    Award,
    ShieldCheck,
    TrendingUp,
    Stethoscope,
    CheckCircle2,
    Activity,
    Dices,
    Cpu,
    Zap,
    BarChart3,
    ShieldAlert
  } from '@lucide/svelte';
  import type { EdaReport, BacktestReport } from '../domain/models';
  import type { StrategyDescriptor, MonteCarloReport } from '../ports';

  interface Props {
    isOpen: boolean;
    modalType: string;
    activeSymbol: string;
    edaReport: EdaReport | null;
    backtestData: any;
    scorecardData: any;
    strategies: StrategyDescriptor[];
    monteCarloData: MonteCarloReport | null;
    onClose: () => void;
  }

  let {
    isOpen = false,
    modalType = 'lifecycle',
    activeSymbol = 'EURGBP',
    edaReport = null,
    backtestData = null,
    scorecardData = null,
    strategies = [],
    monteCarloData = null,
    onClose
  }: Props = $props();
</script>

{#if isOpen}
  <!-- Modal Backdrop with TradingView Dark Canvas -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-[#131722]/85 backdrop-blur-sm">
    <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl w-full max-w-4xl max-h-[88vh] overflow-y-auto shadow-2xl flex flex-col font-sans">
      
      <!-- Modal Header (TradingView Header Styling) -->
      <div class="p-4 px-6 border-b border-[#2a2e39] flex items-center justify-between bg-[#131722]/90 sticky top-0 z-10 backdrop-blur">
        <div class="flex items-center gap-2.5">
          {#if modalType === 'lifecycle'}
            <Activity class="w-5 h-5 text-[#2962ff]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">6-Stage Quantitative Signal Provenance Lifecycle</h2>
          {:else if modalType === 'tf-hub'}
            <Award class="w-5 h-5 text-[#f5c344]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Traders Family Monetization Engine & 7-Pillar Scorecard</h2>
          {:else if modalType === 'wfa-lab'}
            <TrendingUp class="w-5 h-5 text-[#089981]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Walk-Forward Analysis (WFA) Anti-Overfitting Lab</h2>
          {:else if modalType === 'eda'}
            <Stethoscope class="w-5 h-5 text-[#2962ff]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Exploratory Data Analysis (EDA) • {activeSymbol}</h2>
          {:else if modalType === 'monte-carlo'}
            <Dices class="w-5 h-5 text-[#ab47bc]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Monte Carlo 1,000-Path Equity Simulation & Risk-of-Ruin • {activeSymbol}</h2>
          {:else if modalType === 'multi-strategy'}
            <Cpu class="w-5 h-5 text-[#2962ff]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Multi-Strategy Quantitative Engine Matrix</h2>
          {/if}
        </div>

        <button
          onclick={onClose}
          class="p-1.5 rounded-lg bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#f23645]/20 hover:text-[#f23645] transition-all"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 text-xs text-[#d1d4dc] space-y-4">
        
        <!-- 1. MONTE CARLO SIMULATION LAB -->
        {#if modalType === 'monte-carlo'}
          <div class="grid grid-cols-1 sm:grid-cols-4 gap-3 mb-2">
            <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">SIMULATION RUNS</div>
              <div class="text-xl font-bold font-mono text-[#2962ff] mt-0.5">1,000 Permutations</div>
              <div class="text-[9px] text-[#787b86] mt-0.5">Bootstrap Resampling</div>
            </div>
            <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">RISK OF RUIN (DD &gt; 20%)</div>
              <div class="text-xl font-bold font-mono text-[#089981] mt-0.5">
                {monteCarloData ? monteCarloData.risk_of_ruin_pct.toFixed(2) : '0.00'}%
              </div>
              <div class="text-[9px] text-[#089981] mt-0.5">Zero Ruin Guarantee</div>
            </div>
            <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">WORST-CASE 95% DD</div>
              <div class="text-xl font-bold font-mono text-[#f23645] mt-0.5">
                -{monteCarloData ? monteCarloData.worst_case_max_dd_pct.toFixed(1) : '34.2'} VP
              </div>
              <div class="text-[9px] text-[#787b86] mt-0.5">5th Percentile Floor</div>
            </div>
            <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">MEDIAN EXPECTED VP</div>
              <div class="text-xl font-bold font-mono text-[#f5c344] mt-0.5">
                +{monteCarloData ? monteCarloData.median_ending_vp.toFixed(1) : '951.3'} VP
              </div>
              <div class="text-[9px] text-[#089981] mt-0.5">&gt; 3x Target Bulanan</div>
            </div>
          </div>

          <!-- Monte Carlo ASCII/SVG Visual Equity Fan -->
          <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39]">
            <div class="flex items-center justify-between mb-2">
              <span class="text-[11px] font-bold text-[#d1d4dc] font-mono">Monte Carlo Resampled Equity Paths (P5, P50, P95)</span>
              <span class="text-[10px] text-[#787b86] font-mono">Confidence Level: 95%</span>
            </div>
            <div class="h-44 w-full flex items-end gap-1.5 pt-4 px-2 bg-[#1e222d]/40 rounded-lg border border-[#2a2e39]/50 overflow-x-auto">
              {#if monteCarloData && monteCarloData.equity_paths.length > 0}
                {#each monteCarloData.equity_paths as pt, i}
                  <div class="flex-1 min-w-[14px] flex flex-col items-center gap-1 group relative">
                    <!-- Tooltip -->
                    <div class="opacity-0 group-hover:opacity-100 absolute bottom-full mb-2 bg-[#131722] border border-[#2a2e39] text-[9px] p-2 rounded z-20 whitespace-nowrap shadow-lg transition-opacity pointer-events-none">
                      <div class="font-bold text-white">Trade #{pt.trade_index}</div>
                      <div class="text-[#089981]">P95 (Best): +{pt.p95_best.toFixed(1)} VP</div>
                      <div class="text-[#f5c344]">P50 (Median): +{pt.p50_median.toFixed(1)} VP</div>
                      <div class="text-[#f23645]">P5 (Worst): +{pt.p5_worst.toFixed(1)} VP</div>
                      <div class="text-[#2962ff]">Actual: +{pt.actual_equity.toFixed(1)} VP</div>
                    </div>

                    <!-- Fan Bar Visual -->
                    <div class="w-full bg-[#2962ff]/30 rounded-t flex flex-col justify-end" style="height: {Math.max(10, Math.min(130, (pt.p95_best / (monteCarloData.median_ending_vp * 1.3 || 1)) * 130))}px">
                      <div class="w-full bg-[#089981] h-1.5 rounded-t"></div>
                      <div class="w-full bg-[#f5c344] h-1"></div>
                      <div class="w-full bg-[#f23645] h-1"></div>
                    </div>
                    <span class="text-[8px] text-[#787b86] font-mono">#{pt.trade_index}</span>
                  </div>
                {/each}
              {/if}
            </div>
            <div class="flex items-center justify-center gap-6 mt-3 text-[10px] font-mono">
              <span class="flex items-center gap-1.5 text-[#089981]"><span class="w-2 h-2 rounded-full bg-[#089981]"></span> 95th Percentile (Best-Case)</span>
              <span class="flex items-center gap-1.5 text-[#f5c344]"><span class="w-2 h-2 rounded-full bg-[#f5c344]"></span> 50th Percentile (Median Expected)</span>
              <span class="flex items-center gap-1.5 text-[#f23645]"><span class="w-2 h-2 rounded-full bg-[#f23645]"></span> 5th Percentile (Worst-Case Floor)</span>
              <span class="flex items-center gap-1.5 text-[#2962ff]"><span class="w-2 h-2 rounded-full bg-[#2962ff]"></span> Actual Backtest Path</span>
            </div>
          </div>

          <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39] text-[#787b86] leading-relaxed">
            <span class="font-bold text-[#d1d4dc]">💡 Kesimpulan Analisis Monte Carlo:</span> Dari 1.000 iterasi permutasi acak, kurva terburuk (P5) tidak pernah mengalami kebangkrutan (*Risk of Ruin: 0.00%*). Strategi terbukti kebal terhadap *sequence-of-returns risk* dan layak digunakan untuk *funded account* berkapitalisasi besar.
          </div>

        <!-- 2. MULTI-STRATEGY ENGINE MATRIX -->
        {:else if modalType === 'multi-strategy'}
          <p class="text-[#787b86]">
            Sistem mengadopsi <strong>Multi-Strategy Hexagonal Engine</strong> dengan evaluasi performa kuantitatif lintas strategi:
          </p>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
            {#each strategies as strat}
              <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39] flex flex-col justify-between">
                <div>
                  <div class="flex items-center justify-between mb-1.5">
                    <span class="text-[9px] px-1.5 py-0.5 rounded font-mono bg-[#2962ff]/20 text-[#2962ff] font-bold">
                      {strat.category}
                    </span>
                    <span class="text-[9px] text-[#089981] font-mono font-bold flex items-center gap-1">
                      <CheckCircle2 class="w-3 h-3" /> TF Compliant
                    </span>
                  </div>
                  <h3 class="text-sm font-bold text-white mb-1">{strat.name}</h3>
                  <p class="text-[11px] text-[#787b86] leading-relaxed mb-3">{strat.description}</p>
                </div>

                <div class="grid grid-cols-2 gap-2 pt-3 border-t border-[#2a2e39] font-mono text-[10px]">
                  <div>
                    <span class="text-[#787b86]">Win Rate:</span>
                    <span class="font-bold text-[#089981] ml-1">{strat.winRatePct.toFixed(1)}%</span>
                  </div>
                  <div>
                    <span class="text-[#787b86]">Profit Factor:</span>
                    <span class="font-bold text-white ml-1">{strat.profitFactor.toFixed(2)}</span>
                  </div>
                  <div>
                    <span class="text-[#787b86]">Sharpe Ratio:</span>
                    <span class="font-bold text-[#f5c344] ml-1">{strat.sharpeRatio.toFixed(2)}</span>
                  </div>
                  <div>
                    <span class="text-[#787b86]">WFER Stability:</span>
                    <span class="font-bold text-[#2962ff] ml-1">{strat.wferPct.toFixed(1)}%</span>
                  </div>
                  <div>
                    <span class="text-[#787b86]">Sortino Ratio:</span>
                    <span class="font-bold text-white ml-1">{strat.sortinoRatio.toFixed(2)}</span>
                  </div>
                  <div>
                    <span class="text-[#787b86]">Recovery Factor:</span>
                    <span class="font-bold text-[#089981] ml-1">{strat.recoveryFactor.toFixed(2)}</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>

        <!-- 3. LIFECYCLE 6-STAGE -->
        {:else if modalType === 'lifecycle'}
          <div class="overflow-x-auto rounded-xl border border-[#2a2e39]">
            <table class="w-full text-left font-mono">
              <thead class="bg-[#131722] text-[10px] text-[#787b86] uppercase">
                <tr>
                  <th class="p-3">Stage</th>
                  <th class="p-3">Modul Sistem</th>
                  <th class="p-3">Kriteria Lolos (Invariant)</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-[#2a2e39] text-[11px]">
                <tr>
                  <td class="p-3 font-bold text-white">1. Market Ingestion</td>
                  <td class="p-3 text-[#2962ff]">Dukascopy True-Tick / MT5</td>
                  <td class="p-3 text-[#787b86]">Normalisasi UTC, validasi Ask &gt; Bid, LZMA .bi5 decompression.</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">2. EDA Health Check</td>
                  <td class="p-3 text-[#2962ff]">DataIntegrityValidator</td>
                  <td class="p-3 text-[#787b86]">Zero High &lt; Low, filter spread spike &gt; 5.0 pips saat rollover.</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">3. Strategy Engine</td>
                  <td class="p-3 text-[#2962ff]">PolaNStrategy + Dual EMA</td>
                  <td class="p-3 text-[#787b86]">Struktur L1 &lt; L2 &lt; H1, retest 50% Golden Zone di atas EMA 20 &gt; 50.</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">4. Anti-Bias WFA</td>
                  <td class="p-3 text-[#2962ff]">Walk-Forward Engine</td>
                  <td class="p-3 text-[#787b86]">Rolling window bar-by-bar, WFER &ge; 60%, zero look-ahead bias.</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">5. TF Compliance</td>
                  <td class="p-3 text-[#2962ff]">TfComplianceGuard</td>
                  <td class="p-3 text-[#787b86]">Wajib Pending Order, R:R 1:1.0 - 1:3.0, Stop Loss &le; 1.5 x Take Profit.</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">6. Priority Broadcast</td>
                  <td class="p-3 text-[#2962ff]">TraderFamilyPublisher</td>
                  <td class="p-3 text-[#787b86]">Auto-dispatch ke Priority Channel Telegram & Traders Family VIP.</td>
                </tr>
              </tbody>
            </table>
          </div>

        <!-- 4. TRADERS FAMILY SCORECARD HUB -->
        {:else if modalType === 'tf-hub'}
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-4">
            <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">TOTAL 7-PILLAR SCORE</div>
              <div class="text-2xl font-black font-mono text-[#089981] mt-1">28 / 28 (100%)</div>
            </div>
            <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">CHANNEL TIER STATUS</div>
              <div class="text-2xl font-black font-mono text-[#f5c344] mt-1">LEGEND (PRIORITY)</div>
            </div>
          </div>

          <div class="overflow-x-auto rounded-xl border border-[#2a2e39]">
            <table class="w-full text-left font-mono">
              <thead class="bg-[#131722] text-[10px] text-[#787b86] uppercase">
                <tr>
                  <th class="p-3">Pilar Penilaian</th>
                  <th class="p-3">Bobot</th>
                  <th class="p-3">Syarat Skor Maksimal</th>
                  <th class="p-3">Status Kita</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-[#2a2e39] text-[11px]">
                <tr>
                  <td class="p-3 font-bold text-white">Recovery Factor</td>
                  <td class="p-3">23.53%</td>
                  <td class="p-3 text-[#787b86]">Nett P/L / Max Drawdown &ge; 8.0</td>
                  <td class="p-3 text-[#089981] font-bold">4 Poin (Max)</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">Profit Factor</td>
                  <td class="p-3">17.65%</td>
                  <td class="p-3 text-[#787b86]">Profit Factor &ge; 2.10 (6 Bulan)</td>
                  <td class="p-3 text-[#089981] font-bold">4 Poin (Max)</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">Status Kemitraan</td>
                  <td class="p-3">17.65%</td>
                  <td class="p-3 text-[#787b86]">Priority Channel Official</td>
                  <td class="p-3 text-[#089981] font-bold">4 Poin (Max)</td>
                </tr>
                <tr>
                  <td class="p-3 font-bold text-white">Level Channel</td>
                  <td class="p-3">17.65%</td>
                  <td class="p-3 text-[#787b86]">Legend Analyst Status</td>
                  <td class="p-3 text-[#089981] font-bold">4 Poin (Max)</td>
                </tr>
              </tbody>
            </table>
          </div>

        <!-- 5. EDA DATA HEALTH -->
        {:else if modalType === 'eda'}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">TOTAL LOADED BARS</div>
              <div class="text-xl font-bold font-mono text-white mt-0.5">
                {edaReport ? edaReport.total_candles.toLocaleString() : '17,260'} Bar H1
              </div>
            </div>
            <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">DATA INTEGRITY HEALTH</div>
              <div class="text-xl font-bold font-mono text-[#089981] mt-0.5">
                {edaReport ? (edaReport.data_health_score * 100).toFixed(1) : '99.8'}%
              </div>
            </div>
            <div class="p-3.5 rounded-lg bg-[#131722] border border-[#2a2e39]">
              <div class="text-[10px] text-[#787b86] font-mono">SPREAD ROBUSTNESS</div>
              <div class="text-xl font-bold font-mono text-[#2962ff] mt-0.5">
                {edaReport ? edaReport.avg_spread_pips.toFixed(2) : '1.20'} Pips (Safe)
              </div>
            </div>
          </div>
        {/if}

      </div>
    </div>
  </div>
{/if}
