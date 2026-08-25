<script lang="ts">
  import { Calendar, Clock, X, ArrowRight, Zap, Sparkles } from '@lucide/svelte';
  import type { Candle } from '../../domain/models';
  import type { IReplayEnginePort } from '../../ports/IReplayEnginePort';

  interface Props {
    isOpen: boolean;
    candles: Candle[];
    replayEngine: IReplayEnginePort;
    onClose: () => void;
  }

  let { isOpen = false, candles = [], replayEngine, onClose }: Props = $props();

  // Input date string formatted as YYYY-MM-DDTHH:mm
  let selectedDateStr = $state('');
  let previewCandle = $state<Candle | null>(null);

  $effect(() => {
    if (isOpen && candles.length > 0) {
      // Default to 1 month ago
      const lastCandle = candles[candles.length - 1];
      const oneMonthAgoSec = lastCandle.time - (30 * 86400);
      findClosestCandle(oneMonthAgoSec);
    }
  });

  function findClosestCandle(targetTimeSec: number) {
    if (candles.length === 0) return;
    let closest = candles[0];
    let minDiff = Infinity;

    for (let i = 0; i < candles.length; i++) {
      const diff = Math.abs(candles[i].time - targetTimeSec);
      if (diff < minDiff) {
        minDiff = diff;
        closest = candles[i];
      }
    }

    previewCandle = closest;
    const d = new Date(closest.time * 1000);
    // Format to YYYY-MM-DDTHH:mm in client local time for datetime-local input
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    const hours = String(d.getHours()).padStart(2, '0');
    const mins = String(d.getMinutes()).padStart(2, '0');
    selectedDateStr = `${year}-${month}-${day}T${hours}:${mins}`;
  }

  function handlePreset(daysAgo: number) {
    if (candles.length === 0) return;
    const lastCandle = candles[candles.length - 1];
    const targetSec = lastCandle.time - (daysAgo * 86400);
    findClosestCandle(targetSec);
  }

  function handleCustomInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    selectedDateStr = val;
    if (val) {
      // Parse in local browser timezone
      const parsedSec = Math.floor(new Date(val).getTime() / 1000);
      if (!isNaN(parsedSec)) {
        findClosestCandle(parsedSec);
      }
    }
  }

  function handleConfirm() {
    if (previewCandle) {
      replayEngine.startReplayAtTime(previewCandle.time);
      onClose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-150"
    onclick={handleBackdropClick}
    role="presentation"
  >
    <div class="w-full max-w-md bg-[#1e222d] border border-[#2a2e39] rounded-2xl shadow-2xl overflow-hidden font-mono text-[#d1d4dc] animate-in zoom-in-95 duration-150">
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-[#2a2e39] bg-[#131722]/80">
        <div class="flex items-center gap-2">
          <div class="p-2 rounded-lg bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/30">
            <Calendar class="w-4 h-4" />
          </div>
          <div>
            <h3 class="text-sm font-extrabold text-white">Go to Date (Bar Replay)</h3>
            <p class="text-[10px] text-[#787b86]">Lompat ke tanggal masa lalu untuk simulasi bar-by-bar</p>
          </div>
        </div>
        <button
          onclick={onClose}
          class="p-1.5 rounded-lg hover:bg-[#2a2e39] text-[#787b86] hover:text-white transition-colors"
          title="Tutup (Esc)"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-4">
        <!-- Quick Presets -->
        <div>
          <span class="block text-[10px] font-bold text-[#787b86] uppercase tracking-wider mb-2">
            Pilihan Cepat (Presets)
          </span>
          <div class="grid grid-cols-3 gap-2">
            <button
              onclick={() => handlePreset(7)}
              class="px-2.5 py-2 rounded-xl bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-xs font-bold text-left transition-all hover:scale-[1.02] active:scale-[0.98]"
            >
              <div class="text-[10px] text-[#787b86]">1 Minggu</div>
              <div class="text-white font-mono text-[11px]">-7 Hari</div>
            </button>
            <button
              onclick={() => handlePreset(30)}
              class="px-2.5 py-2 rounded-xl bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-xs font-bold text-left transition-all hover:scale-[1.02] active:scale-[0.98]"
            >
              <div class="text-[10px] text-[#787b86]">1 Bulan</div>
              <div class="text-white font-mono text-[11px]">-30 Hari</div>
            </button>
            <button
              onclick={() => handlePreset(90)}
              class="px-2.5 py-2 rounded-xl bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-xs font-bold text-left transition-all hover:scale-[1.02] active:scale-[0.98]"
            >
              <div class="text-[10px] text-[#787b86]">3 Bulan</div>
              <div class="text-white font-mono text-[11px]">-90 Hari</div>
            </button>
            <button
              onclick={() => handlePreset(180)}
              class="px-2.5 py-2 rounded-xl bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-xs font-bold text-left transition-all hover:scale-[1.02] active:scale-[0.98]"
            >
              <div class="text-[10px] text-[#787b86]">6 Bulan</div>
              <div class="text-white font-mono text-[11px]">-180 Hari</div>
            </button>
            <button
              onclick={() => handlePreset(365)}
              class="px-2.5 py-2 rounded-xl bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-xs font-bold text-left transition-all hover:scale-[1.02] active:scale-[0.98]"
            >
              <div class="text-[10px] text-[#787b86]">1 Tahun</div>
              <div class="text-white font-mono text-[11px]">-365 Hari</div>
            </button>
            <button
              onclick={() => handlePreset(730)}
              class="px-2.5 py-2 rounded-xl bg-[#131722] hover:bg-[#2a2e39] border border-[#2a2e39] hover:border-[#2962ff]/60 text-xs font-bold text-left transition-all hover:scale-[1.02] active:scale-[0.98]"
            >
              <div class="text-[10px] text-[#787b86]">2 Tahun</div>
              <div class="text-white font-mono text-[11px]">-730 Hari</div>
            </button>
          </div>
        </div>

        <!-- Custom Date & Time Picker -->
        <div>
          <label for="replay-target-datetime" class="block text-[10px] font-bold text-[#787b86] uppercase tracking-wider mb-2">
            Pilih Tanggal & Jam Spesifik ({typeof Intl !== 'undefined' ? Intl.DateTimeFormat().resolvedOptions().timeZone : 'Waktu Lokal'})
          </label>
          <div class="relative">
            <input
              id="replay-target-datetime"
              type="datetime-local"
              value={selectedDateStr}
              oninput={handleCustomInput}
              class="w-full bg-[#131722] border border-[#2a2e39] focus:border-[#2962ff] focus:ring-1 focus:ring-[#2962ff] rounded-xl px-3.5 py-2.5 text-xs text-white font-mono outline-none transition-all"
            />
          </div>
        </div>

        <!-- Target Candle Preview Badge -->
        {#if previewCandle}
          <div class="p-3 bg-[#131722] border border-[#2a2e39] rounded-xl text-xs space-y-1">
            <div class="flex items-center justify-between text-[10px] text-[#787b86]">
              <span class="font-bold flex items-center gap-1 text-[#2962ff]">
                <Clock class="w-3 h-3" />
                <span>Titik Potong Terkonfirmasi</span>
              </span>
              <span>1H Candle</span>
            </div>
            <div class="flex items-center justify-between pt-1">
              <span class="text-white font-extrabold">
                {new Intl.DateTimeFormat(undefined, {
                  weekday: 'short',
                  day: '2-digit',
                  month: 'short',
                  year: 'numeric',
                  hour: '2-digit',
                  minute: '2-digit',
                  hour12: false,
                  timeZoneName: 'short',
                }).format(new Date(previewCandle.time * 1000))}
              </span>
              <span class="font-black text-[#089981]">
                {previewCandle.close.toFixed(5)}
              </span>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-between px-5 py-4 border-t border-[#2a2e39] bg-[#131722]/60">
        <span class="text-[10px] text-[#787b86]">Shortcut: <kbd class="px-1.5 py-0.5 bg-[#2a2e39] rounded text-white font-mono">Alt + G</kbd></span>
        <div class="flex items-center gap-2">
          <button
            onclick={onClose}
            class="px-4 py-2 rounded-xl bg-[#1e222d] hover:bg-[#2a2e39] text-[#787b86] hover:text-white text-xs font-bold transition-colors"
          >
            Batal
          </button>
          <button
            onclick={handleConfirm}
            class="flex items-center gap-1.5 px-5 py-2 rounded-xl bg-[#2962ff] hover:bg-[#1e53e5] text-white text-xs font-extrabold shadow-lg shadow-[#2962ff]/30 transition-all hover:scale-[1.02] active:scale-[0.98]"
          >
            <span>Mulai Replay</span>
            <ArrowRight class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
