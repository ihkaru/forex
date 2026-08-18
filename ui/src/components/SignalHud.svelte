<script lang="ts">
  import { Radio, Send, Target, ShieldAlert, CheckCircle2, ArrowUpRight, ArrowDownRight, Sparkles } from '@lucide/svelte';
  import type { Signal } from '../domain/models';

  interface Props {
    signal: Signal | null;
    activeSymbol: string;
  }

  let { signal = null, activeSymbol = 'EURGBP' }: Props = $props();

  let isBuy = $derived(signal ? signal.action.includes('BUY') : true);
  let actionLabel = $derived(signal ? signal.action : 'BUY_LIMIT');
  let entryPrice = $derived(signal ? signal.entryPrice.toFixed(5) : '0.85200');
  let stopLoss = $derived(signal ? signal.stopLoss.toFixed(5) : '0.84950');
  let tp1 = $derived(signal ? signal.takeProfit1.toFixed(5) : '0.85700');
  let tp2 = $derived(signal ? signal.takeProfit2.toFixed(5) : '0.85950');
  let rrRatio = $derived(signal ? signal.riskRewardRatio.toFixed(2) : '2.00');
</script>

<div class="flex flex-col gap-4 font-sans">
  <!-- Signal Execution HUD Card (TradingView Styling) -->
  <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-4 shadow-md">
    <div class="flex items-center justify-between pb-3 border-b border-[#2a2e39]">
      <div class="flex items-center gap-2">
        <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 {isBuy ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40'}">
          {#if isBuy}
            <ArrowUpRight class="w-3.5 h-3.5" /> PENDING BUY LIMIT
          {:else}
            <ArrowDownRight class="w-3.5 h-3.5" /> PENDING SELL LIMIT
          {/if}
        </span>
        <span class="text-xs font-mono px-2 py-0.5 rounded bg-[#131722] text-[#787b86] border border-[#2a2e39]">
          R:R 1:{rrRatio}
        </span>
      </div>
      <span class="text-xs font-mono font-bold text-[#f5c344]">
        +60.0 VP Gain
      </span>
    </div>

    <!-- Levels Grid -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 my-3">
      <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
        <div class="text-[10px] text-[#787b86] font-mono font-bold">ENTRY LEVEL</div>
        <div class="text-sm font-black font-mono text-[#2962ff] mt-0.5">{entryPrice}</div>
      </div>
      <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
        <div class="text-[10px] text-[#787b86] font-mono font-bold">STOP LOSS (SL)</div>
        <div class="text-sm font-black font-mono text-[#f23645] mt-0.5">{stopLoss}</div>
      </div>
      <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
        <div class="text-[10px] text-[#787b86] font-mono font-bold">TAKE PROFIT 1</div>
        <div class="text-sm font-black font-mono text-[#089981] mt-0.5">{tp1}</div>
      </div>
      <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
        <div class="text-[10px] text-[#787b86] font-mono font-bold">TAKE PROFIT 2</div>
        <div class="text-sm font-black font-mono text-[#089981] mt-0.5">{tp2}</div>
      </div>
    </div>
  </div>

  <!-- Official Traders Family Channel Broadcast Preview -->
  <div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-4 shadow-md">
    <div class="flex items-center justify-between pb-2 mb-2 border-b border-[#2a2e39]">
      <div class="flex items-center gap-2">
        <Radio class="w-4 h-4 text-[#f5c344]" />
        <h3 class="text-xs font-bold font-mono text-[#d1d4dc]">Official Channel Broadcast</h3>
      </div>
      <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#f5c344]/10 text-[#f5c344] font-bold border border-[#f5c344]/30">
        Traders Family VIP
      </span>
    </div>

    <div class="bg-[#131722] p-3 rounded-lg border border-[#2a2e39] font-mono text-xs text-[#d1d4dc] leading-relaxed">
      <div class="text-[#2962ff] font-black tracking-wide flex items-center gap-1.5 mb-1">
        <Sparkles class="w-3.5 h-3.5" /> FOREX QUANT SIGNAL ALERT
      </div>
      <div class="text-[#2a2e39] text-[10px]">━━━━━━━━━━━━━━━━━━━━━━━━━</div>
      <div class="space-y-0.5 text-[11px] my-1">
        <div>Pair: <b class="text-white">{activeSymbol.slice(0, 3)}/{activeSymbol.slice(3)}</b></div>
        <div>Action: <b class="{isBuy ? 'text-[#089981]' : 'text-[#f23645]'}">{actionLabel}</b></div>
        <div>Entry: <span class="text-[#2962ff] font-bold">{entryPrice}</span></div>
        <div>Stop Loss: <span class="text-[#f23645] font-bold">{stopLoss}</span></div>
        <div>TP 1: <span class="text-[#089981] font-bold">{tp1}</span></div>
        <div>TP 2: <span class="text-[#089981] font-bold">{tp2}</span></div>
        <div>R:R Ratio: <b class="text-white">1:{rrRatio}</b></div>
        <div>Strategy: <span class="text-[#787b86]">TF-Pola-N-Core-v1</span></div>
        <div class="text-[#787b86] text-[10px] mt-1">Note: Retest 50% Golden Zone di atas EMA 20/50</div>
      </div>
      <div class="text-[#2a2e39] text-[10px]">━━━━━━━━━━━━━━━━━━━━━━━━━</div>
      <div class="text-[10px] text-[#089981] font-bold flex items-center gap-1.5 mt-1.5">
        <CheckCircle2 class="w-3.5 h-3.5" /> Broadcast Success (Post ID: tf-live-2026)
      </div>
    </div>
  </div>
</div>
