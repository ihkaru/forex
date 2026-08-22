<script lang="ts">
  import type { DetailedBacktestReport } from '../../ports/ITesterPort';
  import PerformanceSummaryTable from './PerformanceSummaryTable.svelte';
  import ListOfTradesTable from './ListOfTradesTable.svelte';
  import EquityCurveCanvas from './EquityCurveCanvas.svelte';
  import { 
    LayoutDashboard, 
    TableProperties, 
    ListOrdered, 
    TrendingUp, 
    ChevronDown, 
    ChevronUp, 
    ShieldCheck, 
    Layers,
    Award,
    Download
  } from 'lucide-svelte';

  let { 
    report, 
    activeSymbol,
    onSelectTrade = (tradeId: string) => {}
  }: { 
    report: DetailedBacktestReport | null; 
    activeSymbol: string;
    onSelectTrade?: (tradeId: string) => void;
  } = $props();

  let activeTab = $state<'OVERVIEW' | 'PERFORMANCE_SUMMARY' | 'LIST_OF_TRADES' | 'EQUITY_CURVE'>('OVERVIEW');
  let isCollapsed = $state<boolean>(false);

  let decisiveTrades = $derived((report?.winning_trades ?? 0) + (report?.losing_trades ?? 0));
  let decisiveWinRate = $derived(
    decisiveTrades > 0
      ? ((report?.winning_trades ?? 0) / decisiveTrades) * 100
      : (report?.win_rate_percent ?? 0)
  );
  let beTrades = $derived(
    Math.max(
      0,
      (report?.total_trades ?? 0) -
        (report?.winning_trades ?? 0) -
        (report?.losing_trades ?? 0)
    )
  );

  function fmtPips(val: number): string {
    const prefix = val > 0 ? '+' : '';
    return `${prefix}${val.toFixed(1)} pips`;
  }

  function downloadJsonReport() {
    if (!report) return;
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(report, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', `${activeSymbol}_H1_backtest_report.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  }
</script>

<div class="w-full bg-[#1e222d] border-t border-[#2a2e39] text-[#d1d4dc] transition-all flex flex-col">
  <!-- Header Bar -->
  <div class="flex items-center justify-between px-4 py-2 bg-[#171b26] border-b border-[#2a2e39] text-xs">
    <div class="flex items-center gap-6">
      <div class="flex items-center gap-2">
        <Layers class="w-4 h-4 text-[#2962ff]" />
        <span class="font-bold tracking-wide uppercase text-white font-sans">
          Strategy Tester • {activeSymbol} (H1)
        </span>
        {#if report}
          <span class="px-2 py-0.5 rounded text-[10px] font-bold {report.is_tf_qualified ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40'}">
            {report.is_tf_qualified ? 'TF QUALIFIED (300+ VP)' : 'IN PROGRESS'}
          </span>
        {/if}
      </div>

      <!-- Tab Buttons -->
      {#if !isCollapsed}
        <div class="flex items-center gap-1 bg-[#131722] p-1 rounded border border-[#2a2e39]/60">
          <button
            onclick={() => (activeTab = 'OVERVIEW')}
            class="flex items-center gap-1.5 px-3 py-1 rounded text-xs transition-colors {activeTab === 'OVERVIEW' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
          >
            <LayoutDashboard class="w-3.5 h-3.5" />
            Overview
          </button>
          <button
            onclick={() => (activeTab = 'PERFORMANCE_SUMMARY')}
            class="flex items-center gap-1.5 px-3 py-1 rounded text-xs transition-colors {activeTab === 'PERFORMANCE_SUMMARY' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
          >
            <TableProperties class="w-3.5 h-3.5" />
            Performance Summary
          </button>
          <button
            onclick={() => (activeTab = 'LIST_OF_TRADES')}
            class="flex items-center gap-1.5 px-3 py-1 rounded text-xs transition-colors {activeTab === 'LIST_OF_TRADES' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
          >
            <ListOrdered class="w-3.5 h-3.5" />
            List of Trades ({report?.trades?.length || 0})
          </button>
          <button
            onclick={() => (activeTab = 'EQUITY_CURVE')}
            class="flex items-center gap-1.5 px-3 py-1 rounded text-xs transition-colors {activeTab === 'EQUITY_CURVE' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
          >
            <TrendingUp class="w-3.5 h-3.5" />
            Equity Curve
          </button>
        </div>
      {/if}
    </div>

    <!-- Right Controls -->
    <div class="flex items-center gap-3">
      {#if report && !isCollapsed}
        <div class="flex items-center gap-4 text-xs font-mono pr-2">
          <div>
            <span class="text-[#787b86]">Net PnL:</span>
            <span class="font-bold ml-1 {report.total_raw_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
              {fmtPips(report.total_raw_pips)}
            </span>
          </div>
          <div>
            <span class="text-[#787b86]">VP:</span>
            <span class="font-bold ml-1 {report.total_valued_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
              {report.total_valued_pips >= 0 ? '+' : ''}{report.total_valued_pips.toFixed(1)} VP
            </span>
          </div>
          <div>
            <span class="text-[#787b86]">PF:</span>
            <span class="font-bold ml-1 text-[#2962ff]">{report.profit_factor.toFixed(2)}</span>
          </div>

          <button
            onclick={downloadJsonReport}
            class="flex items-center gap-1 px-2.5 py-1 rounded bg-[#131722] hover:bg-[#2a2e39] text-[#d1d4dc] hover:text-white border border-[#2a2e39] transition-all font-mono text-[11px] ml-2"
            title="Download Full Backtest & Trades JSON"
          >
            <Download class="w-3 h-3 text-[#2962ff]" /> Export JSON
          </button>
        </div>
      {/if}

      <button
        onclick={() => (isCollapsed = !isCollapsed)}
        class="p-1 text-[#787b86] hover:text-white rounded hover:bg-[#2a2e39] transition-colors"
        title={isCollapsed ? 'Expand Strategy Tester' : 'Collapse Strategy Tester'}
      >
        {#if isCollapsed}
          <ChevronUp class="w-4 h-4" />
        {:else}
          <ChevronDown class="w-4 h-4" />
        {/if}
      </button>
    </div>
  </div>

  <!-- Content Area -->
  {#if !isCollapsed}
    <div class="p-4 bg-[#131722] min-h-[260px] max-h-[360px] overflow-y-auto">
      {#if !report}
        <div class="flex items-center justify-center h-48 text-[#787b86] text-xs">
          Loading comprehensive deep backtesting report...
        </div>
      {:else if activeTab === 'OVERVIEW'}
        <!-- Bento KPI Grid -->
        <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-3 mb-4">
          <!-- Net Profit -->
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <span class="text-[11px] text-[#787b86] font-medium block">Total Net Profit</span>
            <span class="text-base font-bold font-mono {report.total_raw_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
              {fmtPips(report.total_raw_pips)}
            </span>
            <span class="text-[10px] text-[#787b86] block mt-0.5">
              Valued: {report.total_valued_pips >= 0 ? '+' : ''}{report.total_valued_pips.toFixed(1)} VP
            </span>
          </div>

          <!-- Profit Factor -->
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <span class="text-[11px] text-[#787b86] font-medium block">Profit Factor</span>
            <span class="text-base font-bold font-mono {report.profit_factor >= 1.5 ? 'text-[#089981]' : 'text-[#f23645]'}">
              {report.profit_factor.toFixed(2)}
            </span>
            <span class="text-[10px] text-[#787b86] block mt-0.5">
              Gross: +{report.gross_profit_pips.toFixed(1)} / -{report.gross_loss_pips.toFixed(1)}
            </span>
          </div>

          <!-- Win Rate (Decisive & Breakdown) -->
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <div class="flex items-center justify-between">
              <span class="text-[11px] text-[#787b86] font-medium block">Win Rate (Decisive)</span>
              <span class="text-[9px] font-mono px-1.5 py-0.2 rounded bg-[#089981]/20 text-[#089981] font-bold">
                W vs L
              </span>
            </div>
            <span class="text-base font-bold font-mono {decisiveWinRate >= 50 || report.profit_factor >= 1.5 ? 'text-[#089981]' : 'text-[#f23645]'}">
              {decisiveWinRate.toFixed(1)}%
            </span>
            <span class="text-[10px] text-[#787b86] block mt-0.5" title="Total Sample: {report.winning_trades} Wins ({report.win_rate_percent.toFixed(1)}%), {report.losing_trades} Losses, {beTrades} Breakeven">
              {report.winning_trades} W / {report.losing_trades} L • {beTrades} BE
            </span>
          </div>

          <!-- Max Drawdown -->
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <span class="text-[11px] text-[#787b86] font-medium block">Max Drawdown</span>
            <span class="text-base font-bold font-mono text-[#f23645]">
              -{report.max_drawdown_pips.toFixed(1)} pips
            </span>
            <span class="text-[10px] text-[#787b86] block mt-0.5">
              Peak-to-Trough Decline
            </span>
          </div>

          <!-- Recovery Factor -->
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <span class="text-[11px] text-[#787b86] font-medium block">Recovery Factor</span>
            <span class="text-base font-bold font-mono {report.recovery_factor >= 2.0 ? 'text-[#089981]' : 'text-[#d1d4dc]'}">
              {report.recovery_factor.toFixed(2)}
            </span>
            <span class="text-[10px] text-[#787b86] block mt-0.5">
              Net PnL / Max DD (TF Metric)
            </span>
          </div>

          <!-- Total Trades -->
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <span class="text-[11px] text-[#787b86] font-medium block">Total Trades</span>
            <span class="text-base font-bold font-mono text-white">
              {report.total_trades}
            </span>
            <span class="text-[10px] text-[#787b86] block mt-0.5">
              Settled in Sample
            </span>
          </div>
        </div>

        <!-- Quick Mini Equity & Performance Summary Preview -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <h4 class="text-xs font-bold text-[#d1d4dc] mb-2 flex items-center gap-1.5">
              <TrendingUp class="w-3.5 h-3.5 text-[#2962ff]" />
              Equity Curve Preview
            </h4>
            <EquityCurveCanvas data={report.equity_curve} />
          </div>
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-3">
            <h4 class="text-xs font-bold text-[#d1d4dc] mb-2 flex items-center gap-1.5">
              <TableProperties class="w-3.5 h-3.5 text-[#089981]" />
              Key Statistics
            </h4>
            {#if report.summary}
              <PerformanceSummaryTable summary={report.summary} />
            {/if}
          </div>
        </div>
      {:else if activeTab === 'PERFORMANCE_SUMMARY'}
        {#if report.summary}
          <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-4">
            <PerformanceSummaryTable summary={report.summary} />
          </div>
        {/if}
      {:else if activeTab === 'LIST_OF_TRADES'}
        <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg overflow-hidden">
          <ListOfTradesTable trades={report.trades} {onSelectTrade} />
        </div>
      {:else if activeTab === 'EQUITY_CURVE'}
        <div class="bg-[#1e222d] border border-[#2a2e39] rounded-lg p-4">
          <EquityCurveCanvas data={report.equity_curve} />
        </div>
      {/if}
    </div>
  {/if}
</div>
