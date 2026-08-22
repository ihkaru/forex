<script lang="ts">
  import { ChevronLeft, ChevronRight } from 'lucide-svelte';

  let { 
    trades = [], 
    onSelectTrade = (tradeId: string) => {} 
  }: { 
    trades: any[]; 
    onSelectTrade?: (tradeId: string) => void;
  } = $props();

  let filterResult = $state<'ALL' | 'WIN' | 'LOSS'>('ALL');
  let filterDirection = $state<'ALL' | 'BUY' | 'SELL'>('ALL');
  let pageSize = $state<number>(25);
  let currentPage = $state<number>(1);
  let selectedId = $state<string | null>(null);

  const filteredTrades = $derived(
    trades.filter((t) => {
      const isWin = (t.pnl_pips || 0) > 0 || t.is_win;
      if (filterResult === 'WIN' && !isWin) return false;
      if (filterResult === 'LOSS' && isWin) return false;

      const act = String(t.action || '').toUpperCase();
      if (filterDirection === 'BUY' && !act.includes('BUY')) return false;
      if (filterDirection === 'SELL' && !act.includes('SELL')) return false;

      return true;
    })
  );

  const totalPages = $derived(
    Math.max(1, Math.ceil(filteredTrades.length / pageSize))
  );

  const pagedTrades = $derived(
    filteredTrades.slice((currentPage - 1) * pageSize, currentPage * pageSize)
  );

  const filterSummary = $derived.by(() => {
    let winCount = 0;
    let netPnl = 0;
    let netVp = 0;
    for (const t of filteredTrades) {
      const pnl = Number(t.pnl_pips || 0);
      const vp = Number(t.valued_pips || 0);
      netPnl += pnl;
      netVp += vp;
      if (pnl > 0 || t.is_win) winCount++;
    }
    const winRate = filteredTrades.length > 0 ? (winCount / filteredTrades.length) * 100 : 0;
    return {
      winCount,
      lossCount: filteredTrades.length - winCount,
      winRate,
      netPnl,
      netVp
    };
  });

  function formatDate(ts: number | string): string {
    if (!ts) return '-';
    if (typeof ts === 'string') return ts.replace('T', ' ').substring(0, 16);
    const d = new Date(ts * 1000);
    return d.toISOString().replace('T', ' ').substring(0, 16);
  }

  function handleRowClick(trade: any) {
    selectedId = trade.id;
    onSelectTrade(trade.id);
  }
</script>

<div class="flex flex-col w-full h-full text-xs">
  <!-- Rich Filter & Controls Bar -->
  <div class="flex flex-wrap items-center justify-between gap-3 px-4 py-2 border-b border-[#2a2e39] bg-[#1e222d]">
    <div class="flex flex-wrap items-center gap-3">
      <!-- Result Filter Pills -->
      <div class="flex items-center gap-1 bg-[#131722] p-1 rounded-md border border-[#2a2e39]">
        <button
          onclick={() => { filterResult = 'ALL'; currentPage = 1; }}
          class="px-2.5 py-0.5 rounded text-[11px] transition-colors {filterResult === 'ALL' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
        >
          All ({trades.length})
        </button>
        <button
          onclick={() => { filterResult = 'WIN'; currentPage = 1; }}
          class="px-2.5 py-0.5 rounded text-[11px] transition-colors {filterResult === 'WIN' ? 'bg-[#089981]/20 text-[#089981] font-bold border border-[#089981]/40' : 'text-[#787b86] hover:text-[#089981]'}"
        >
          Wins ({trades.filter((t) => (t.pnl_pips || 0) > 0 || t.is_win).length})
        </button>
        <button
          onclick={() => { filterResult = 'LOSS'; currentPage = 1; }}
          class="px-2.5 py-0.5 rounded text-[11px] transition-colors {filterResult === 'LOSS' ? 'bg-[#f23645]/20 text-[#f23645] font-bold border border-[#f23645]/40' : 'text-[#787b86] hover:text-[#f23645]'}"
        >
          Losses ({trades.filter((t) => (t.pnl_pips || 0) <= 0 && !t.is_win).length})
        </button>
      </div>

      <!-- Direction Filter Pills -->
      <div class="flex items-center gap-1 bg-[#131722] p-1 rounded-md border border-[#2a2e39]">
        <button
          onclick={() => { filterDirection = 'ALL'; currentPage = 1; }}
          class="px-2 py-0.5 rounded text-[11px] transition-colors {filterDirection === 'ALL' ? 'bg-[#2a2e39] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
        >
          Any Direction
        </button>
        <button
          onclick={() => { filterDirection = 'BUY'; currentPage = 1; }}
          class="px-2 py-0.5 rounded text-[11px] transition-colors {filterDirection === 'BUY' ? 'bg-[#2962ff]/20 text-[#2962ff] font-bold border border-[#2962ff]/40' : 'text-[#787b86] hover:text-[#2962ff]'}"
        >
          Long / Buy
        </button>
        <button
          onclick={() => { filterDirection = 'SELL'; currentPage = 1; }}
          class="px-2 py-0.5 rounded text-[11px] transition-colors {filterDirection === 'SELL' ? 'bg-[#e040fb]/20 text-[#e040fb] font-bold border border-[#e040fb]/40' : 'text-[#787b86] hover:text-[#e040fb]'}"
        >
          Short / Sell
        </button>
      </div>

      <!-- Filtered Aggregate Summary Badge -->
      <div class="hidden xl:flex items-center gap-3 text-[11px] text-[#787b86] font-mono border-l border-[#2a2e39] pl-3">
        <span>Matched: <strong class="text-white">{filteredTrades.length}</strong></span>
        <span>Win Rate: <strong class="{filterSummary.winRate >= 50 ? 'text-[#089981]' : 'text-[#f23645]'}">{filterSummary.winRate.toFixed(1)}%</strong></span>
        <span>Filtered PnL: <strong class="{filterSummary.netPnl >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">{filterSummary.netPnl >= 0 ? '+' : ''}{filterSummary.netPnl.toFixed(1)} pips ({filterSummary.netVp >= 0 ? '+' : ''}{filterSummary.netVp.toFixed(1)} VP)</strong></span>
      </div>
    </div>

    <!-- Pagination Controls -->
    <div class="flex items-center gap-2 text-[11px]">
      <span class="text-[#787b86]">Rows per page:</span>
      <select 
        bind:value={pageSize}
        onchange={() => (currentPage = 1)}
        class="bg-[#131722] text-[#d1d4dc] border border-[#2a2e39] rounded px-1.5 py-0.5 font-mono"
      >
        <option value={10}>10</option>
        <option value={25}>25</option>
        <option value={50}>50</option>
        <option value={100}>100</option>
        <option value={500}>All</option>
      </select>

      <span class="text-[#787b86] font-mono ml-2">
        Page {currentPage} of {totalPages}
      </span>

      <button
        disabled={currentPage <= 1}
        onclick={() => (currentPage = Math.max(1, currentPage - 1))}
        class="p-1 rounded bg-[#131722] hover:bg-[#2a2e39] disabled:opacity-30 disabled:hover:bg-[#131722] transition-colors"
        title="Previous Page"
      >
        <ChevronLeft class="w-3.5 h-3.5" />
      </button>

      <button
        disabled={currentPage >= totalPages}
        onclick={() => (currentPage = Math.min(totalPages, currentPage + 1))}
        class="p-1 rounded bg-[#131722] hover:bg-[#2a2e39] disabled:opacity-30 disabled:hover:bg-[#131722] transition-colors"
        title="Next Page"
      >
        <ChevronRight class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>

  <!-- Table Body -->
  <div class="overflow-y-auto max-h-72 w-full">
    <table class="w-full text-xs text-left border-collapse">
      <thead class="sticky top-0 bg-[#1e222d] text-[#787b86] font-medium border-b border-[#2a2e39] z-10">
        <tr>
          <th class="py-2 px-3">#</th>
          <th class="py-2 px-3">Type</th>
          <th class="py-2 px-3">Date/Time In</th>
          <th class="py-2 px-3 text-right">Price In</th>
          <th class="py-2 px-3">Date/Time Out</th>
          <th class="py-2 px-3 text-right">Price Out</th>
          <th class="py-2 px-3 text-right">P&L (Pips)</th>
          <th class="py-2 px-3 text-right">Valued Pips</th>
          <th class="py-2 px-3 text-center">Status</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-[#2a2e39]/40 font-mono text-[11px]">
        {#if pagedTrades.length === 0}
          <tr>
            <td colspan="9" class="py-8 text-center text-[#787b86]">No trades match the selected filters.</td>
          </tr>
        {:else}
          {#each pagedTrades as trade, idx}
            {@const isWin = (trade.pnl_pips || 0) > 0 || trade.is_win}
            <tr
              onclick={() => handleRowClick(trade)}
              class="cursor-pointer transition-colors {selectedId === trade.id ? 'bg-[#2962ff]/20 border-l-2 border-[#2962ff]' : 'hover:bg-[#2a2e39]/40'}"
            >
              <td class="py-1.5 px-3 text-[#787b86]">{(currentPage - 1) * pageSize + idx + 1}</td>
              <td class="py-1.5 px-3 font-sans font-bold {String(trade.action).includes('Buy') ? 'text-[#2962ff]' : 'text-[#e040fb]'}">
                {trade.action}
              </td>
              <td class="py-1.5 px-3 text-[#d1d4dc]">{formatDate(trade.open_time)}</td>
              <td class="py-1.5 px-3 text-right text-[#d1d4dc]">{Number(trade.open_price).toFixed(5)}</td>
              <td class="py-1.5 px-3 text-right text-[#d1d4dc]">{formatDate(trade.close_time)}</td>
              <td class="py-1.5 px-3 text-right text-[#d1d4dc]">{Number(trade.close_price).toFixed(5)}</td>
              <td class="py-1.5 px-3 text-right font-bold {isWin ? 'text-[#089981]' : 'text-[#f23645]'}">
                {Number(trade.pnl_pips) >= 0 ? '+' : ''}{Number(trade.pnl_pips).toFixed(1)}
              </td>
              <td class="py-1.5 px-3 text-right font-bold {isWin ? 'text-[#089981]' : 'text-[#f23645]'}">
                {Number(trade.valued_pips) >= 0 ? '+' : ''}{Number(trade.valued_pips).toFixed(1)} VP
              </td>
              <td class="py-1.5 px-3 text-center">
                {#if isWin}
                  <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-[#089981]/20 text-[#089981] border border-[#089981]/40">
                    TP HIT
                  </span>
                {:else}
                  <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40">
                    SL HIT
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>
