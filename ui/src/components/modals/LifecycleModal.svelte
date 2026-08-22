<script lang="ts">
</script>

<div class="overflow-x-auto rounded-xl border border-[#2a2e39]">
  <table class="w-full text-left font-mono">
    <thead class="bg-[#131722] text-[10px] text-[#787b86] uppercase">
      <tr>
        <th class="p-3">Stage</th>
        <th class="p-3">Modul Sistem</th>
        <th class="p-3">Kriteria Lolos (Invariant)</th>
      </tr>
    </thead>
    <tbody class="divide-y divide-[#2a2e39] text-[11px]">
      <tr>
        <td class="p-3 font-bold text-white">1. Market Ingestion</td>
        <td class="p-3 text-[#2962ff]">Dukascopy True-Tick / MT5</td>
        <td class="p-3 text-[#787b86]">Normalisasi UTC, validasi Ask &gt; Bid, LZMA .bi5 decompression.</td>
      </tr>
      <tr>
        <td class="p-3 font-bold text-white">2. EDA Health Check</td>
        <td class="p-3 text-[#2962ff]">DataIntegrityValidator</td>
        <td class="p-3 text-[#787b86]">Zero High &lt; Low, filter spread spike &gt; 5.0 pips saat rollover.</td>
      </tr>
      <tr>
        <td class="p-3 font-bold text-white">3. Strategy Engine</td>
        <td class="p-3 text-[#2962ff]">PolaNStrategy + Dual EMA</td>
        <td class="p-3 text-[#787b86]">Struktur L1 &lt; L2 &lt; H1, retest 50% Golden Zone di atas EMA 20 &gt; 50.</td>
      </tr>
      <tr>
        <td class="p-3 font-bold text-white">4. Anti-Bias WFA</td>
        <td class="p-3 text-[#2962ff]">Walk-Forward Engine</td>
        <td class="p-3 text-[#787b86]">Rolling window bar-by-bar, WFER &ge; 60%, zero look-ahead bias.</td>
      </tr>
      <tr>
        <td class="p-3 font-bold text-white">5. TF Compliance</td>
        <td class="p-3 text-[#2962ff]">TfComplianceGuard</td>
        <td class="p-3 text-[#787b86]">Wajib Pending Order, R:R 1:1.0 - 1:3.0, Stop Loss &le; 1.5 x Take Profit.</td>
      </tr>
      <tr>
        <td class="p-3 font-bold text-white">6. Priority Broadcast</td>
        <td class="p-3 text-[#2962ff]">TraderFamilyPublisher</td>
        <td class="p-3 text-[#787b86]">Auto-dispatch ke Priority Channel Telegram &amp; Traders Family VIP.</td>
      </tr>
    </tbody>
  </table>
</div>
