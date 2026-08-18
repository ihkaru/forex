<script lang="ts">
  let { 
    trades = [], 
    onSelectTrade = (tradeId: string) => {} 
  }: { 
    trades: any[]; 
    onSelectTrade?: (tradeId: string) => void;
  } = $props();

  let filterType = $state<'ALL' | 'WIN' | 'LOSS'>('ALL');
  let selectedId = $state<string | null>(null);

  const filteredTrades = $derived(
    trades.filter((t) => {
      if (filterType === 'WIN') return t.is_win;
      if (filterType === 'LOSS') return !t.is_win;
      return true;
    })
  );

  function formatDate(ts: number): string {
    if (!ts) return '-';
    const d = new Date(ts * 1000);
    return d.toISOString().replace('T', ' ').substring(0, 16);
  }

  function handleRowClick(trade: any) {
    selectedId = trade.id;
    onSelectTrade(trade.id);
  }
</script>

<div class="flex flex-col w-full h-full">
  <!-- Controls Bar -->
  <div class="flex items-center justify-between px-4 py-2 border-b border-[#2a2e39] bg-[#1e222d] text-xs">
    <div class="flex items-center gap-2">
      <span class="text-[#787b86] font-medium">Filter:</span>
      <button
        onclick={() => (filterType = 'ALL')}
        class="px-2.5 py-1 rounded transition-colors {filterType === 'ALL' ? 'bg-[#2962ff] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
      >
        All ({trades.len || trades.length})
      </button>
      <button
        onclick={() => (filterType = 'WIN')}
        class="px-2.5 py-1 rounded transition-colors {filterType === 'WIN' ? 'bg-[#089981]/20 text-[#089981] font-bold border border-[#089981]/40' : 'text-[#787b86] hover:text-[#089981]'}"
      >
        Wins ({trades.filter((t) => t.is_win).length})
      </button>
      <button
        onclick={() => (filterType = 'LOSS')}
        class="px-2.5 py-1 rounded transition-colors {filterType === 'LOSS' ? 'bg-[#f23645]/20 text-[#f23645] font-bold border border-[#f23645]/40' : 'text-[#787b86] hover:text-[#f23645]'}"
      >
        Losses ({trades.filter((t) => !t.is_win).length})
      </button>
    </div>
    <span class="text-[#787b86] text-[11px]">Click a trade to jump to bar on chart</span>
  </div>

  <!-- Table Body -->
  <div class="overflow-y-auto max-h-72 w-full">
    <table class="w-full text-xs text-left border-collapse">
      <thead class="sticky top-0 bg-[#1e222d] text-[#787b86] font-medium border-b border-[#2a2e39]">
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
        {#if filteredTrades.length === 0}
          <tr>
            <td colspan="9" class="py-8 text-center text-[#787b86]">No trades recorded in this period.</td>
          </tr>
        {:else}
          {#each filteredTrades as trade, idx}
            <tr
              onclick={() => handleRowClick(trade)}
              class="cursor-pointer transition-colors {selectedId === trade.id ? 'bg-[#2962ff]/20 border-l-2 border-[#2962ff]' : 'hover:bg-[#2a2e39]/40'}"
            >
              <td class="py-1.5 px-3 text-[#787b86]">{idx + 1}</td>
              <td class="py-1.5 px-3 font-sans font-bold {trade.action.includes('Buy') ? 'text-[#2962ff]' : 'text-[#e040fb]'}">
                {trade.action}
              </td>
              <td class="py-1.5 px-3 text-[#d1d4dc]">{formatDate(trade.open_time)}</td>
              <td class="py-1.5 px-3 text-right text-[#d1d4dc]">{Number(trade.open_price).toFixed(5)}</td>
              <td class="py-1.5 px-3 text-[#d1d4dc]">{formatDate(trade.close_time)}</td>
              <td class="py-1.5 px-3 text-right text-[#d1d4dc]">{Number(trade.close_price).toFixed(5)}</td>
              <td class="py-1.5 px-3 text-right font-bold {trade.pnl_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
                {trade.pnl_pips >= 0 ? '+' : ''}{Number(trade.pnl_pips).toFixed(1)}
              </td>
              <td class="py-1.5 px-3 text-right font-bold {trade.valued_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
                {trade.valued_pips >= 0 ? '+' : ''}{Number(trade.valued_pips).toFixed(1)} VP
              </td>
              <td class="py-1.5 px-3 text-center">
                {#if trade.is_win}
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
