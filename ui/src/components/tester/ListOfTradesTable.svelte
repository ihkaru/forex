<script lang="ts">
  import { 
    ChevronLeft, 
    ChevronRight, 
    Search, 
    X, 
    ArrowUp, 
    ArrowDown, 
    ArrowUpDown,
    Filter
  } from 'lucide-svelte';

  let { 
    trades = [], 
    onSelectTrade = (tradeId: string) => {} 
  }: { 
    trades: any[]; 
    onSelectTrade?: (tradeId: string) => void;
  } = $props();

  let filterResult = $state<'ALL' | 'WIN' | 'LOSS'>('ALL');
  let filterDirection = $state<'ALL' | 'BUY' | 'SELL'>('ALL');
  let filterThreshold = $state<'ALL' | 'GT_100' | 'GT_500' | 'LT_MINUS_200'>('ALL');
  let searchQuery = $state<string>('');
  let pageSize = $state<number>(25);
  let currentPage = $state<number>(1);
  let selectedId = $state<string | null>(null);

  // Column Sorting State
  type SortKey = 'idx' | 'action' | 'open_time' | 'open_price' | 'close_time' | 'close_price' | 'pnl_pips' | 'valued_pips' | 'status';
  let sortKey = $state<SortKey>('close_time');
  let sortAsc = $state<boolean>(false);

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
    } else {
      sortKey = key;
      sortAsc = (key === 'open_time' || key === 'close_time' || key === 'pnl_pips' || key === 'valued_pips') ? false : true;
    }
  }

  const filteredAndSortedTrades = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    
    // Parse comparison operators from search query if present
    let opType: 'NONE' | 'GT' | 'GTE' | 'LT' | 'LTE' | 'PNL_GT' | 'PNL_LT' | 'VP_GT' | 'VP_LT' | 'PRICE_GT' | 'PRICE_LT' | 'HOURS_GT' | 'HOURS_LT' = 'NONE';
    let opNum = 0;

    if (q.startsWith('>=')) {
      const n = parseFloat(q.substring(2));
      if (!isNaN(n)) { opType = 'GTE'; opNum = n; }
    } else if (q.startsWith('<=')) {
      const n = parseFloat(q.substring(2));
      if (!isNaN(n)) { opType = 'LTE'; opNum = n; }
    } else if (q.startsWith('>')) {
      const n = parseFloat(q.substring(1));
      if (!isNaN(n)) { opType = 'GT'; opNum = n; }
    } else if (q.startsWith('<')) {
      const n = parseFloat(q.substring(1));
      if (!isNaN(n)) { opType = 'LT'; opNum = n; }
    } else if (q.startsWith('pnl>')) {
      const n = parseFloat(q.substring(4));
      if (!isNaN(n)) { opType = 'PNL_GT'; opNum = n; }
    } else if (q.startsWith('pnl<')) {
      const n = parseFloat(q.substring(4));
      if (!isNaN(n)) { opType = 'PNL_LT'; opNum = n; }
    } else if (q.startsWith('vp>')) {
      const n = parseFloat(q.substring(3));
      if (!isNaN(n)) { opType = 'VP_GT'; opNum = n; }
    } else if (q.startsWith('vp<')) {
      const n = parseFloat(q.substring(3));
      if (!isNaN(n)) { opType = 'VP_LT'; opNum = n; }
    } else if (q.startsWith('price>')) {
      const n = parseFloat(q.substring(6));
      if (!isNaN(n)) { opType = 'PRICE_GT'; opNum = n; }
    } else if (q.startsWith('price<')) {
      const n = parseFloat(q.substring(6));
      if (!isNaN(n)) { opType = 'PRICE_LT'; opNum = n; }
    } else if (q.startsWith('hours>') || q.startsWith('dur>')) {
      const n = parseFloat(q.split('>')[1]);
      if (!isNaN(n)) { opType = 'HOURS_GT'; opNum = n; }
    } else if (q.startsWith('hours<') || q.startsWith('dur<')) {
      const n = parseFloat(q.split('<')[1]);
      if (!isNaN(n)) { opType = 'HOURS_LT'; opNum = n; }
    }

    // 1. Filter
    const list = trades.filter((t) => {
      const pnl = Number(t.pnl_pips || 0);
      const vp = Number(t.valued_pips || 0);
      const isWin = pnl > 0 || t.is_win;

      // Result filter
      if (filterResult === 'WIN' && !isWin) return false;
      if (filterResult === 'LOSS' && isWin) return false;

      // Direction filter
      const act = String(t.action || '').toUpperCase();
      if (filterDirection === 'BUY' && !act.includes('BUY')) return false;
      if (filterDirection === 'SELL' && !act.includes('SELL')) return false;

      // Threshold filter
      if (filterThreshold === 'GT_100' && pnl <= 100) return false;
      if (filterThreshold === 'GT_500' && pnl <= 500) return false;
      if (filterThreshold === 'LT_MINUS_200' && pnl >= -200) return false;

      // Search / Operator comparison
      if (opType === 'GT') {
        if (pnl <= opNum && vp <= opNum) return false;
      } else if (opType === 'GTE') {
        if (pnl < opNum && vp < opNum) return false;
      } else if (opType === 'LT') {
        if (pnl >= opNum && vp >= opNum) return false;
      } else if (opType === 'LTE') {
        if (pnl > opNum && vp > opNum) return false;
      } else if (opType === 'PNL_GT') {
        if (pnl <= opNum) return false;
      } else if (opType === 'PNL_LT') {
        if (pnl >= opNum) return false;
      } else if (opType === 'VP_GT') {
        if (vp <= opNum) return false;
      } else if (opType === 'VP_LT') {
        if (vp >= opNum) return false;
      } else if (opType === 'PRICE_GT') {
        if (Number(t.open_price) <= opNum && Number(t.close_price) <= opNum) return false;
      } else if (opType === 'PRICE_LT') {
        if (Number(t.open_price) >= opNum && Number(t.close_price) >= opNum) return false;
      } else if (opType === 'HOURS_GT') {
        if (Number(t.duration_hours || 0) <= opNum) return false;
      } else if (opType === 'HOURS_LT') {
        if (Number(t.duration_hours || 0) >= opNum) return false;
      } else if (q) {
        const idMatch = String(t.id || '').toLowerCase().includes(q);
        const actMatch = act.toLowerCase().includes(q);
        const reasonMatch = String(t.exit_reason || '').toLowerCase().includes(q);
        const priceInMatch = String(t.open_price || '').includes(q);
        const priceOutMatch = String(t.close_price || '').includes(q);
        const pnlMatch = String(t.pnl_pips || '').includes(q);
        const dateInMatch = formatDate(t.open_time).toLowerCase().includes(q);
        const dateOutMatch = formatDate(t.close_time).toLowerCase().includes(q);

        if (!idMatch && !actMatch && !reasonMatch && !priceInMatch && !priceOutMatch && !pnlMatch && !dateInMatch && !dateOutMatch) {
          return false;
        }
      }

      return true;
    });

    // 2. Sort
    list.sort((a, b) => {
      let cmp = 0;
      switch (sortKey) {
        case 'action':
          cmp = String(a.action || '').localeCompare(String(b.action || ''));
          break;
        case 'open_time':
          cmp = (Number(a.open_time) || 0) - (Number(b.open_time) || 0);
          break;
        case 'open_price':
          cmp = (Number(a.open_price) || 0) - (Number(b.open_price) || 0);
          break;
        case 'close_time':
          cmp = (Number(a.close_time) || 0) - (Number(b.close_time) || 0);
          break;
        case 'close_price':
          cmp = (Number(a.close_price) || 0) - (Number(b.close_price) || 0);
          break;
        case 'pnl_pips':
          cmp = (Number(a.pnl_pips) || 0) - (Number(b.pnl_pips) || 0);
          break;
        case 'valued_pips':
          cmp = (Number(a.valued_pips) || 0) - (Number(b.valued_pips) || 0);
          break;
        case 'status':
          cmp = Number(a.is_win || 0) - Number(b.is_win || 0);
          break;
        default:
          cmp = 0;
      }
      return sortAsc ? cmp : -cmp;
    });

    return list;
  });

  const totalPages = $derived(
    Math.max(1, Math.ceil(filteredAndSortedTrades.length / pageSize))
  );

  const pagedTrades = $derived(
    filteredAndSortedTrades.slice((currentPage - 1) * pageSize, currentPage * pageSize)
  );

  const filterSummary = $derived.by(() => {
    let winCount = 0;
    let netPnl = 0;
    let netVp = 0;
    for (const t of filteredAndSortedTrades) {
      const pnl = Number(t.pnl_pips || 0);
      const vp = Number(t.valued_pips || 0);
      netPnl += pnl;
      netVp += vp;
      if (pnl > 0 || t.is_win) winCount++;
    }
    const winRate = filteredAndSortedTrades.length > 0 ? (winCount / filteredAndSortedTrades.length) * 100 : 0;
    return {
      winCount,
      lossCount: filteredAndSortedTrades.length - winCount,
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
  <!-- Rich Filter, Search & Controls Bar -->
  <div class="flex flex-wrap items-center justify-between gap-3 px-4 py-2 border-b border-[#2a2e39] bg-[#1e222d]">
    <div class="flex flex-wrap items-center gap-3">
      <!-- Search Input with Comparison Operators support -->
      <div class="relative flex items-center">
        <Search class="w-3.5 h-3.5 text-[#787b86] absolute left-2.5 pointer-events-none" />
        <input
          type="text"
          bind:value={searchQuery}
          oninput={() => (currentPage = 1)}
          placeholder="Search price, date, >100, <0..."
          title="Support operator perbandingan: >1000, <0, pnl>50, vp>100, hours>24, price>0.86"
          class="bg-[#131722] text-[#d1d4dc] placeholder-[#787b86] text-[11px] pl-8 pr-7 py-1 rounded-md border border-[#2a2e39] focus:outline-none focus:border-[#2962ff] w-52 font-mono"
        />
        {#if searchQuery}
          <button
            onclick={() => { searchQuery = ''; currentPage = 1; }}
            class="absolute right-2 text-[#787b86] hover:text-white"
            title="Clear search"
          >
            <X class="w-3 h-3" />
          </button>
        {/if}
      </div>

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

      <!-- Quick Threshold Comparison Pills (> / <) -->
      <div class="hidden 2xl:flex items-center gap-1 bg-[#131722] p-1 rounded-md border border-[#2a2e39]">
        <span class="text-[10px] text-[#787b86] px-1 font-mono">Threshold:</span>
        <button
          onclick={() => { filterThreshold = 'ALL'; currentPage = 1; }}
          class="px-1.5 py-0.5 rounded text-[10px] transition-colors {filterThreshold === 'ALL' ? 'bg-[#2a2e39] text-white font-bold' : 'text-[#787b86] hover:text-[#d1d4dc]'}"
        >
          All
        </button>
        <button
          onclick={() => { filterThreshold = 'GT_100'; currentPage = 1; }}
          class="px-1.5 py-0.5 rounded text-[10px] transition-colors {filterThreshold === 'GT_100' ? 'bg-[#089981]/20 text-[#089981] font-bold border border-[#089981]/40' : 'text-[#787b86] hover:text-[#089981]'}"
        >
          &gt; +100 pips
        </button>
        <button
          onclick={() => { filterThreshold = 'GT_500'; currentPage = 1; }}
          class="px-1.5 py-0.5 rounded text-[10px] transition-colors {filterThreshold === 'GT_500' ? 'bg-[#089981]/20 text-[#089981] font-bold border border-[#089981]/40' : 'text-[#787b86] hover:text-[#089981]'}"
        >
          &gt; +500 pips
        </button>
        <button
          onclick={() => { filterThreshold = 'LT_MINUS_200'; currentPage = 1; }}
          class="px-1.5 py-0.5 rounded text-[10px] transition-colors {filterThreshold === 'LT_MINUS_200' ? 'bg-[#f23645]/20 text-[#f23645] font-bold border border-[#f23645]/40' : 'text-[#787b86] hover:text-[#f23645]'}"
        >
          &lt; -200 pips
        </button>
      </div>

      <!-- Filtered Aggregate Summary Badge -->
      <div class="hidden xl:flex items-center gap-3 text-[11px] text-[#787b86] font-mono border-l border-[#2a2e39] pl-3">
        <span>Matched: <strong class="text-white">{filteredAndSortedTrades.length}</strong></span>
        <span>Win Rate: <strong class="{filterSummary.winRate >= 50 ? 'text-[#089981]' : 'text-[#f23645]'}">{filterSummary.winRate.toFixed(1)}%</strong></span>
        <span>Filtered PnL: <strong class="{filterSummary.netPnl >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">{filterSummary.netPnl >= 0 ? '+' : ''}{filterSummary.netPnl.toFixed(1)} pips ({filterSummary.netVp >= 0 ? '+' : ''}{filterSummary.netVp.toFixed(1)} VP)</strong></span>
      </div>
    </div>

    <!-- Pagination Controls -->
    <div class="flex items-center gap-2 text-[11px]">
      <span class="text-[#787b86]">Rows:</span>
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

  <!-- Table Body with Interactive Sortable Headers -->
  <div class="overflow-y-auto max-h-72 w-full">
    <table class="w-full text-xs text-left border-collapse">
      <thead class="sticky top-0 bg-[#1e222d] text-[#787b86] font-medium border-b border-[#2a2e39] z-10 select-none">
        <tr>
          <th class="py-2 px-3">#</th>
          
          <!-- Column: Type -->
          <th 
            onclick={() => toggleSort('action')}
            class="py-2 px-3 cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center gap-1">
              <span>Type</span>
              {#if sortKey === 'action'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>

          <!-- Column: Date/Time In -->
          <th 
            onclick={() => toggleSort('open_time')}
            class="py-2 px-3 cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center gap-1">
              <span>Date/Time In</span>
              {#if sortKey === 'open_time'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>

          <!-- Column: Price In -->
          <th 
            onclick={() => toggleSort('open_price')}
            class="py-2 px-3 text-right cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center justify-end gap-1">
              <span>Price In</span>
              {#if sortKey === 'open_price'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>

          <!-- Column: Date/Time Out -->
          <th 
            onclick={() => toggleSort('close_time')}
            class="py-2 px-3 cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center gap-1">
              <span>Date/Time Out</span>
              {#if sortKey === 'close_time'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>

          <!-- Column: Price Out -->
          <th 
            onclick={() => toggleSort('close_price')}
            class="py-2 px-3 text-right cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center justify-end gap-1">
              <span>Price Out</span>
              {#if sortKey === 'close_price'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>

          <!-- Column: P&L (Pips) -->
          <th 
            onclick={() => toggleSort('pnl_pips')}
            class="py-2 px-3 text-right cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center justify-end gap-1">
              <span>P&L (Pips)</span>
              {#if sortKey === 'pnl_pips'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>

          <!-- Column: Valued Pips -->
          <th 
            onclick={() => toggleSort('valued_pips')}
            class="py-2 px-3 text-right cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center justify-end gap-1">
              <span>Valued Pips</span>
              {#if sortKey === 'valued_pips'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>

          <!-- Column: Status -->
          <th 
            onclick={() => toggleSort('status')}
            class="py-2 px-3 text-center cursor-pointer hover:text-white transition-colors"
          >
            <div class="flex items-center justify-center gap-1">
              <span>Status</span>
              {#if sortKey === 'status'}
                {#if sortAsc}<ArrowUp class="w-3 h-3 text-[#2962ff]" />{:else}<ArrowDown class="w-3 h-3 text-[#2962ff]" />{/if}
              {:else}
                <ArrowUpDown class="w-3 h-3 opacity-30" />
              {/if}
            </div>
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-[#2a2e39]/40 font-mono text-[11px]">
        {#if pagedTrades.length === 0}
          <tr>
            <td colspan="9" class="py-8 text-center text-[#787b86]">No trades match the selected search & filter criteria.</td>
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
              <td class="py-1.5 px-3 text-[#d1d4dc]">{formatDate(trade.close_time)}</td>
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
