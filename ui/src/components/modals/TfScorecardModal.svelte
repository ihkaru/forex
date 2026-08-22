<script lang="ts">
  import type { TfScorecardReport } from '../../ports';

  interface Props {
    scorecardData: TfScorecardReport | null;
  }

  let { scorecardData = null }: Props = $props();

  let score = $derived(scorecardData?.total_score ?? 20);
  let maxScore = $derived(scorecardData?.max_score ?? 28);
  let pct = $derived(Math.round((score / maxScore) * 100));
  let tier = $derived(scorecardData?.channel_level ?? 'MASTER_PRIORITY');
  let revShare = $derived(
    scorecardData?.revenue_sharing_eligible
      ? score >= 24
        ? '80% (Legend Tier)'
        : score >= 18
          ? '70% (Master Tier)'
          : score >= 12
            ? '60% (Pro Tier)'
            : '50% (Silver Tier)'
      : 'Not Eligible'
  );
  let pillars = $derived(scorecardData?.pillars ?? []);
</script>

<div class="space-y-4">
  <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-4">
    <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono">TOTAL 7-PILLAR SCORE</div>
      <div class="text-2xl font-black font-mono text-[#089981] mt-1">{score} / {maxScore} ({pct}%)</div>
      <div class="text-[10px] text-[#787b86] mt-0.5 font-mono">Bobot Penilaian Resmi TF</div>
    </div>
    <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono">CHANNEL TIER STATUS</div>
      <div class="text-2xl font-black font-mono text-[#2962ff] mt-1">{tier.replace('_', ' ')}</div>
      <div class="text-[10px] text-[#089981] mt-0.5 font-mono font-bold">Priority Verified</div>
    </div>
    <div class="p-4 rounded-xl bg-[#131722] border border-[#2a2e39]">
      <div class="text-[10px] text-[#787b86] font-mono">REVENUE SHARING ELIGIBLE</div>
      <div class="text-2xl font-black font-mono text-[#f5c344] mt-1">{revShare}</div>
      <div class="text-[10px] text-[#787b86] mt-0.5 font-mono">Bagi Hasil Subscriber</div>
    </div>
  </div>

  <div class="overflow-x-auto rounded-xl border border-[#2a2e39]">
    <table class="w-full text-left font-mono">
      <thead class="bg-[#131722] text-[10px] text-[#787b86] uppercase">
        <tr>
          <th class="p-3">Kode</th>
          <th class="p-3">Pilar Penilaian</th>
          <th class="p-3">Bobot</th>
          <th class="p-3">Nilai Riil &amp; Benchmark</th>
          <th class="p-3">Skor Dicapai</th>
          <th class="p-3">Status</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-[#2a2e39] text-[11px]">
        {#each pillars as p}
          {@const isMax = p.score === p.max_score}
          {@const isZero = p.score === 0}
          <tr>
            <td class="p-3 font-bold text-[#2962ff]">{p.code}</td>
            <td class="p-3 font-bold text-white">{p.name}</td>
            <td class="p-3 text-[#787b86]">{p.weight_pct}%</td>
            <td class="p-3 text-[#d1d4dc]">{p.value_label || p.benchmark_rule}</td>
            <td class="p-3 font-bold {isMax ? 'text-[#089981]' : isZero ? 'text-[#f23645]' : 'text-[#f5c344]'}">
              {p.score} / {p.max_score} Poin
            </td>
            <td class="p-3 font-mono text-[10px]">
              <span class="px-2 py-0.5 rounded font-bold {isMax ? 'bg-[#089981]/15 text-[#089981] border border-[#089981]/30' : isZero ? 'bg-[#f23645]/15 text-[#f23645] border border-[#f23645]/30' : 'bg-[#f5c344]/15 text-[#f5c344] border border-[#f5c344]/30'}">
                {p.status}
              </span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
