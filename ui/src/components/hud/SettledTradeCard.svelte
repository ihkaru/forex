<script lang="ts">
  import { Award } from '@lucide/svelte';
  import type { SettledTradeState } from '../../domain/models';

  interface Props {
    settledTrade: SettledTradeState;
  }

  let { settledTrade }: Props = $props();
</script>

<div class="bg-[#1e222d] border border-[#2a2e39] rounded-xl p-3.5 shadow-xl flex flex-col gap-3 animate-in zoom-in-95 duration-150">
  <!-- Settled Header -->
  <div class="flex items-center justify-between pb-2 border-b border-[#2a2e39]">
    <span class="px-2.5 py-1 rounded-lg text-xs font-mono font-bold flex items-center gap-1.5 {settledTrade.isWin ? 'bg-[#089981]/20 text-[#089981] border border-[#089981]/40' : 'bg-[#f23645]/20 text-[#f23645] border border-[#f23645]/40'}">
      <Award class="w-3.5 h-3.5" />
      <span>{settledTrade.isWin ? '🎉 TAKE PROFIT HIT (SETTLED)' : '🛑 STOP LOSS HIT (SETTLED)'}</span>
    </span>
    <span class="text-xs font-mono font-bold {settledTrade.isWin ? 'text-[#089981]' : 'text-[#f23645]'}">
      {settledTrade.valuedPips > 0 ? '+' : ''}{settledTrade.valuedPips.toFixed(1)} VP
    </span>
  </div>

  <!-- Outcome Summary Card -->
  <div class="grid grid-cols-2 gap-2 text-xs font-mono">
    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <span class="text-[10px] text-[#787b86]">Hasil Realisasi:</span>
      <div class="font-bold text-base mt-0.5 {settledTrade.isWin ? 'text-[#089981]' : 'text-[#f23645]'}">
        {settledTrade.pnlPips > 0 ? '+' : ''}{settledTrade.pnlPips.toFixed(1)} Pips
      </div>
    </div>
    <div class="bg-[#131722] p-2.5 rounded-lg border border-[#2a2e39]">
      <span class="text-[10px] text-[#787b86]">Alasan Penutupan:</span>
      <div class="font-bold text-white mt-0.5">
        {settledTrade.exitReason}
      </div>
    </div>
  </div>

  <div class="text-[10px] font-mono text-[#787b86] text-center pt-1 border-t border-[#2a2e39]/60">
    Trade selesai dieksekusi secara deterministik. Sistem kembali memindai pola berikutnya.
  </div>
</div>
