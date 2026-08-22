<script lang="ts">
  import type { EdaReport } from '../../domain/models';

  interface Props {
    edaReport: EdaReport | null;
  }

  let { edaReport = null }: Props = $props();
</script>

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
