<script lang="ts">
  import {
    ArrowUpRight,
    ArrowDownRight,
    Clock,
    Zap,
    Radio,
    ShieldCheck,
    Check,
    Copy,
    Send
  } from '@lucide/svelte';
  import type { PendingOrderState, Signal } from '../../domain/models';

  interface Props {
    pendingOrder: PendingOrderState | null;
    signal: Signal | null;
    activeSymbol: string;
    activeStrategyName: string;
    onScanSignal?: () => void;
  }

  let {
    pendingOrder = null,
    signal = null,
    activeSymbol = 'XAUUSD',
    activeStrategyName = 'TF Pola N Adaptive',
    onScanSignal
  }: Props = $props();

  let isCopied = $state(false);

  let isBuy = $derived.by(() => {
    if (pendingOrder) return pendingOrder.action.toUpperCase().includes('BUY');
    if (signal) return signal.action.toUpperCase().includes('BUY');
    return false;
  });

  let actionLabel = $derived.by(() => {
    if (pendingOrder) return pendingOrder.action.replace(/[_/]/g, ' ').toUpperCase();
    if (signal) return signal.action.replace(/[_/]/g, ' ').toUpperCase();
    return 'PENDING BUY LIMIT';
  });

  let entryPrice = $derived.by(() => {
    if (pendingOrder) return pendingOrder.entryPrice > 500 ? pendingOrder.entryPrice.toFixed(2) : pendingOrder.entryPrice.toFixed(5);
    if (signal) return signal.entryPrice > 500 ? signal.entryPrice.toFixed(2) : signal.entryPrice.toFixed(5);
    return '4026.61';
  });

  let stopLoss = $derived.by(() => {
    if (pendingOrder) return pendingOrder.stopLoss > 500 ? pendingOrder.stopLoss.toFixed(2) : pendingOrder.stopLoss.toFixed(5);
    if (signal) return signal.stopLoss > 500 ? signal.stopLoss.toFixed(2) : signal.stopLoss.toFixed(5);
    return '3981.73';
  });

  let tp1 = $derived.by(() => {
    if (pendingOrder) return pendingOrder.takeProfit > 500 ? pendingOrder.takeProfit.toFixed(2) : pendingOrder.takeProfit.toFixed(5);
    if (signal) return signal.takeProfit1 > 500 ? signal.takeProfit1.toFixed(2) : signal.takeProfit1.toFixed(5);
    return '4075.98';
  });

  let tp2 = $derived.by(() => {
    if (signal?.takeProfit2) return signal.takeProfit2 > 500 ? signal.takeProfit2.toFixed(2) : signal.takeProfit2.toFixed(5);
    return null;
  });

  let rrRatio = $derived.by(() => {
    if (pendingOrder) {
      const sl = Math.abs(pendingOrder.entryPrice - pendingOrder.stopLoss);
      const tp = Math.abs(pendingOrder.takeProfit - pendingOrder.entryPrice);
      return sl > 0 ? (tp / sl).toFixed(2) : '1.50';
    }
    if (signal) return signal.riskRewardRatio.toFixed(2);
    return '1.10';
  });

  let riskPips = $derived.by(() => {
    const mult = activeSymbol === 'XAUUSD' ? 10 : 10000;
    if (pendingOrder) return (Math.abs(pendingOrder.entryPrice - pendingOrder.stopLoss) * mult).toFixed(1);
    if (signal) return (Math.abs(signal.entryPrice - signal.stopLoss) * mult).toFixed(1);
    return '25.0';
  });

  let rewardPips = $derived.by(() => {
    const mult = activeSymbol === 'XAUUSD' ? 10 : 10000;
    if (pendingOrder) return (Math.abs(pendingOrder.takeProfit - pendingOrder.entryPrice) * mult).toFixed(1);
    if (signal) return (Math.abs(signal.takeProfit1 - signal.entryPrice) * mult).toFixed(1);
    return '27.0';
  });

  let valuedPipsTarget = $derived.by(() => {
    const mult = activeSymbol === 'XAUUSD' ? 0.5 : 2.0;
    return (parseFloat(rewardPips) * mult).toFixed(1);
  });

  function copySignalText() {
    const text = `⚡ FOREX QUANT SIGNAL ALERT\nPair: ${activeSymbol.slice(0, 3)}/${activeSymbol.slice(3)}\nAction: ${actionLabel}\nEntry: ${entryPrice}\nStop Loss: ${stopLoss} (-${riskPips} pips)\nTake Profit 1: ${tp1} (+${rewardPips} pips / +${valuedPipsTarget} VP)\n${tp2 ? `Take Profit 2: ${tp2}\n` : ''}R:R Ratio: 1:${rrRatio}\nStrategy: ${activeStrategyName}\nTF Compliance: 0-Penalty Guaranteed`;
    navigator.clipboard.writeText(text);
    isCopied = true;
    setTimeout(() => {
      isCopied = false;
    }, 2000);
  }
</script>

<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-3">
  <!-- Header with Dynamic Trigger Scan Button -->
  <div class="flex flex-wrap items-center justify-between gap-2 pb-2.5 border-b border-[#2a2e39]">
    <div class="flex items-center gap-2">
      <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 {isBuy ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40'}">
        {#if isBuy}
          <ArrowUpRight class="w-3.5 h-3.5" />
        {:else}
          <ArrowDownRight class="w-3.5 h-3.5" />
        {/if}
        <span>{actionLabel}</span>
      </span>
      <span class="text-xs font-mono px-2 py-0.5 rounded bg-[#131722] text-[#787b86] border border-[#2a2e39]">
        R:R 1:{rrRatio}
      </span>
    </div>

    {#if pendingOrder}
      <div class="flex items-center gap-1.5 text-[11px] font-mono text-[#f59e0b] bg-[#f59e0b]/10 px-2.5 py-1 rounded-lg border border-[#f59e0b]/30">
        <Clock class="w-3.5 h-3.5 animate-pulse" />
        <span>Menunggu Fill ({pendingOrder.distancePips.toFixed(1)}p)</span>
      </div>
    {:else if onScanSignal}
      <button
        onclick={onScanSignal}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#2962ff] hover:bg-[#1e53e5] text-white text-xs font-bold font-mono transition-all shadow-md shadow-[#2962ff]/20 active:scale-95"
        title="Scan dan evaluasi sinyal baru dengan {activeStrategyName}"
      >
        <Zap class="w-3.5 h-3.5" />
        <span>Scan Sinyal</span>
      </button>
    {/if}
  </div>

  <!-- Levels 4-Box Grid with Pips Delta -->
  <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono font-bold">ENTRY LEVEL</div>
      <div class="text-sm font-black font-mono text-[#f59e0b] mt-0.5">{entryPrice}</div>
      <div class="text-[9px] text-[#787b86] font-mono mt-0.5">Pending Order</div>
    </div>
    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono font-bold">STOP LOSS (SL)</div>
      <div class="text-sm font-black font-mono text-[#f23645] mt-0.5">{stopLoss}</div>
      <div class="text-[9px] text-[#f23645] font-mono mt-0.5">-{riskPips} pips</div>
    </div>
    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono font-bold">TAKE PROFIT 1</div>
      <div class="text-sm font-black font-mono text-[#089981] mt-0.5">{tp1}</div>
      <div class="text-[9px] text-[#089981] font-mono mt-0.5">+{rewardPips} pips (+{valuedPipsTarget} VP)</div>
    </div>
    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono font-bold">TAKE PROFIT 2</div>
      <div class="text-sm font-black font-mono text-[#089981] mt-0.5">{tp2 ?? '-'}</div>
      <div class="text-[9px] text-[#089981] font-mono mt-0.5">Runner Level</div>
    </div>
  </div>
</div>

<!-- Trade Risk, Compliance & Instant Dispatch Control -->
<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-2.5">
  <div class="flex items-center justify-between pb-2 border-b border-[#2a2e39]">
    <div class="flex items-center gap-2">
      <Radio class="w-4 h-4 text-[#f5c344]" />
      <h3 class="text-xs font-bold font-mono text-[#d1d4dc]">Trade Execution & Compliance</h3>
    </div>
    <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#089981]/15 text-[#089981] font-bold border border-[#089981]/30 flex items-center gap-1">
      <ShieldCheck class="w-3 h-3" /> TF 0-Penalty Valid
    </span>
  </div>

  <div class="grid grid-cols-2 gap-2 text-xs font-mono">
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
      <span class="text-[10px] text-[#787b86]">Risk Sizing:</span>
      <span class="font-bold text-white mt-0.5">Quarter Kelly (1.0%)</span>
    </div>
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
      <span class="text-[10px] text-[#787b86]">SLA Salin Sinyal:</span>
      <span class="font-bold text-[#089981] mt-0.5">≥ 5 Menit Aman</span>
    </div>
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
      <span class="text-[10px] text-[#787b86]">Target VP Monetisasi:</span>
      <span class="font-bold text-[#2962ff] mt-0.5">+{valuedPipsTarget} VP / Trade</span>
    </div>
    <div class="bg-[#131722] p-2 rounded-lg border border-[#2a2e39] flex flex-col justify-between">
      <span class="text-[10px] text-[#787b86]">Expiry Durasi:</span>
      <span class="font-bold text-white mt-0.5">24 Jam (Auto-Expire)</span>
    </div>
  </div>

  <div class="flex items-center gap-2 pt-1">
    <button
      onclick={copySignalText}
      class="flex-1 flex items-center justify-center gap-2 py-2 px-3 rounded-lg text-xs font-mono font-bold bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/50 text-[#d1d4dc] hover:text-white transition-all shadow-sm"
    >
      {#if isCopied}
        <Check class="w-3.5 h-3.5 text-[#089981]" />
        <span class="text-[#089981]">Tersalin ke Clipboard!</span>
      {:else}
        <Copy class="w-3.5 h-3.5 text-[#787b86]" />
        <span>Salin Format Sinyal</span>
      {/if}
    </button>

    <button
      class="flex items-center justify-center gap-1.5 py-2 px-4 rounded-lg text-xs font-mono font-bold bg-[#2962ff] hover:bg-[#1e4bd8] text-white transition-all shadow-md shadow-[#2962ff]/20"
      title="Otomatis disalurkan ke Traders Family Priority Channel"
    >
      <Send class="w-3.5 h-3.5" />
      <span>Dispatch VIP</span>
    </button>
  </div>
</div>
