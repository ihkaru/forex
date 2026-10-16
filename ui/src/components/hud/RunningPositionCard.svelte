<script lang="ts">
  import { Activity, Clock } from '@lucide/svelte';
  import type { RunningPositionState } from '../../domain/models';

  interface Props {
    runningPosition: RunningPositionState;
  }

  let { runningPosition }: Props = $props();

  let isPosBuy = $derived(runningPosition.action.toUpperCase().includes('BUY'));
  let isProfit = $derived(runningPosition.floatingPips >= 0);
</script>

<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-xl flex flex-col gap-3">
  <!-- Running Header -->
  <div class="flex items-center justify-between pb-2.5 border-b border-[#2a2e39]">
    <div class="flex items-center gap-2">
      <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-black flex items-center gap-1.5 bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/40">
        <Activity class="w-3.5 h-3.5 animate-pulse text-[#2962ff]" />
        <span>IN-FLIGHT POSITION</span>
      </span>
      <span class="text-[11px] font-mono font-bold px-2 py-0.5 rounded {isPosBuy ? 'bg-[#089981]/20 text-[#089981]' : 'bg-[#f23645]/20 text-[#f23645]'}">
        {runningPosition.action}
      </span>
    </div>
    <div class="flex items-center gap-1 text-[11px] font-mono text-[#787b86]">
      <Clock class="w-3 h-3" />
      <span>{runningPosition.heldBarsCount} Bar ({runningPosition.heldBarsCount}h)</span>
    </div>
  </div>

  <!-- Floating PnL Hero Card -->
  <div class="p-3.5 rounded-xl border flex items-center justify-between transition-all {isProfit ? 'bg-[#089981]/15 border-[#089981]/40' : 'bg-[#f23645]/15 border-[#f23645]/40'}">
    <div>
      <div class="text-[10px] font-mono font-bold uppercase tracking-wider {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
        Floating Unrealized PnL
      </div>
      <div class="text-2xl font-black font-mono mt-0.5 {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
        {isProfit ? '+' : ''}{runningPosition.floatingPips.toFixed(1)} Pips
      </div>
    </div>
    <div class="text-right">
      <div class="text-[10px] font-mono text-[#787b86] uppercase">Valued Pips</div>
      <div class="text-lg font-black font-mono {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
        {isProfit ? '+' : ''}{runningPosition.floatingValuedPips.toFixed(1)} VP
      </div>
    </div>
  </div>

  <!-- Distance to TP Progress Gauge -->
  <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39] space-y-1.5">
    <div class="flex items-center justify-between text-[10px] font-mono">
      <span class="text-[#787b86]">Target Progress:</span>
      <span class="font-bold {isProfit ? 'text-[#089981]' : 'text-[#f23645]'}">
        {Math.min(100, Math.max(0, runningPosition.progressToTpPct)).toFixed(0)}% to Take Profit
      </span>
    </div>
    <div class="w-full bg-[#2a2e39] h-2 rounded-full overflow-hidden">
      <div
        class="h-full transition-all duration-300 {isProfit ? 'bg-[#089981]' : 'bg-[#f23645]'}"
        style="width: {Math.min(100, Math.max(5, runningPosition.progressToTpPct))}%"
      ></div>
    </div>
  </div>

  <!-- Key Position Levels 4-Box Grid -->
  <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
      <div class="text-[9px] text-[#787b86] font-mono">OPEN PRICE</div>
      <div class="text-xs font-black font-mono text-white mt-0.5">
        {runningPosition.openPrice > 500 ? runningPosition.openPrice.toFixed(2) : runningPosition.openPrice.toFixed(5)}
      </div>
    </div>
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
      <div class="text-[9px] text-[#787b86] font-mono">CURRENT PRICE</div>
      <div class="text-xs font-black font-mono text-[#2962ff] mt-0.5">
        {runningPosition.currentPrice > 500 ? runningPosition.currentPrice.toFixed(2) : runningPosition.currentPrice.toFixed(5)}
      </div>
    </div>
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
      <div class="text-[9px] text-[#787b86] font-mono">TAKE PROFIT</div>
      <div class="text-xs font-black font-mono text-[#089981] mt-0.5">
        {runningPosition.takeProfit > 500 ? runningPosition.takeProfit.toFixed(2) : runningPosition.takeProfit.toFixed(5)}
      </div>
    </div>
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39]">
      <div class="text-[9px] text-[#787b86] font-mono">STOP LOSS</div>
      <div class="text-xs font-black font-mono text-[#f23645] mt-0.5">
        {runningPosition.stopLoss > 500 ? runningPosition.stopLoss.toFixed(2) : runningPosition.stopLoss.toFixed(5)}
      </div>
    </div>
  </div>
</div>
