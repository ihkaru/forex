<script lang="ts">
  import {
    Radar,
    Zap,
    TrendingUp,
    TrendingDown,
    Activity,
    Sparkles,
    ShieldCheck
  } from '@lucide/svelte';
  import type { MarketScanContext } from '../../domain/models';
  import type { PointInTimeComplianceState } from '../../ports';

  interface Props {
    activeSymbol: string;
    scanContext: MarketScanContext | null;
    complianceState: PointInTimeComplianceState | null;
    onScanSignal?: () => void;
  }

  let {
    activeSymbol = 'XAUUSD',
    scanContext = null,
    complianceState = null,
    onScanSignal
  }: Props = $props();
</script>

<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-3">
  <!-- Radar Header -->
  <div class="flex items-center justify-between pb-2.5 border-b border-[#2a2e39]">
    <div class="flex items-center gap-2">
      <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 bg-[#2962ff]/15 text-[#2962ff] border border-[#2962ff]/30">
        <Radar class="w-3.5 h-3.5 animate-spin text-[#2962ff]" style="animation-duration: 4s;" />
        <span>SCANNING MARKET</span>
      </span>
      <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-[#131722] text-[#787b86] border border-[#2a2e39]">
        {activeSymbol}
      </span>
    </div>

    {#if onScanSignal}
      <button
        onclick={onScanSignal}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[#2962ff] hover:bg-[#1e53e5] text-white text-xs font-bold font-mono transition-all shadow-md shadow-[#2962ff]/20 active:scale-95"
        title="Scan market sekarang"
      >
        <Zap class="w-3.5 h-3.5" />
        <span>Scan Market</span>
      </button>
    {/if}
  </div>

  <!-- Tactical Market Radar Grid -->
  <div class="grid grid-cols-2 gap-2 font-mono text-xs">
    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <span class="text-[10px] text-[#787b86]">Tren EMA Alignment:</span>
      <div class="font-bold flex items-center gap-1 mt-0.5 {scanContext?.trend === 'BULLISH' ? 'text-[#089981]' : scanContext?.trend === 'BEARISH' ? 'text-[#f23645]' : 'text-[#f5c344]'}">
        {#if scanContext?.trend === 'BULLISH'}
          <TrendingUp class="w-3.5 h-3.5" /> Bullish Alignment
        {:else if scanContext?.trend === 'BEARISH'}
          <TrendingDown class="w-3.5 h-3.5" /> Bearish Alignment
        {:else}
          <Activity class="w-3.5 h-3.5" /> Neutral / Ranging
        {/if}
      </div>
    </div>

    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <span class="text-[10px] text-[#787b86]">RSI Pullback Window:</span>
      <div class="font-bold text-white mt-0.5">
        {scanContext ? `${scanContext.rsi.toFixed(1)} (Pullback)` : '48.2 (Golden Window)'}
      </div>
    </div>

    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <span class="text-[10px] text-[#787b86]">Session Liquidity:</span>
      <div class="font-bold mt-0.5 {scanContext?.isSessionActive ?? true ? 'text-[#089981]' : 'text-[#787b86]'}">
        {scanContext?.sessionName ?? 'London / NY Active'}
      </div>
    </div>

    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <span class="text-[10px] text-[#787b86]">Status Formasi:</span>
      <div class="font-bold text-[#f5c344] mt-0.5">
        {scanContext?.waitingReason ?? 'Menunggu Retracement 61.8%'}
      </div>
    </div>
  </div>

  <!-- Tactical Status Footer -->
  <div class="p-2.5 rounded-lg bg-[#131722] border border-[#2a2e39]/60 text-[10px] font-mono text-[#787b86] flex items-center gap-2">
    <Sparkles class="w-4 h-4 text-[#2962ff] flex-shrink-0" />
    <span>Standby: Order hanya akan diposting jika seluruh 5 filter institusional Pola N terkonfirmasi secara deterministik.</span>
  </div>
</div>

<!-- TF Rule Invariant Compliance Guarantee (Point-In-Time Live Guard) -->
<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-md flex flex-col gap-2 font-mono text-xs">
  <div class="flex items-center justify-between pb-1.5 border-b border-[#2a2e39]">
    <div class="flex items-center gap-1.5 text-[#d1d4dc] font-bold">
      <ShieldCheck class="w-3.5 h-3.5 text-[#089981]" />
      <span>TF Compliance Policy</span>
    </div>
    <span class="text-[10px] {complianceState?.isCompliant ?? true ? 'text-[#089981]' : 'text-[#f23645]'} font-bold">
      {complianceState?.isCompliant ?? true ? '100% Deterministic' : '⚠️ Quota Alert'}
    </span>
  </div>
  <div class="grid grid-cols-2 gap-2 text-[10px] text-[#787b86] pt-1">
    <div class="flex items-center gap-1">
      <span class="text-[#089981]">✓</span> Dilarang Instant Order
    </div>
    <div class="flex items-center gap-1">
      <span class="text-[#089981]">✓</span> Batas R:R 1:1.0 - 1:3.0
    </div>
    <div class="flex items-center gap-1">
      <span class="text-[#089981]">✓</span> SLA Salin Sinyal ≥ 5 Menit
    </div>
    <div class="flex items-center gap-1">
      <span class="{(complianceState?.activeSignalsOnPair ?? 0) > 0 ? 'text-[#f5c344]' : 'text-[#089981]'}">●</span>
      <span>Kuota: <strong class="{(complianceState?.activeSignalsOnPair ?? 0) > 0 ? 'text-white' : 'text-[#787b86]'}">{complianceState?.activeSignalsOnPair ?? 0}/{complianceState?.maxSignalsPerPair ?? 2} Sinyal Aktif</strong></span>
    </div>
  </div>
</div>
