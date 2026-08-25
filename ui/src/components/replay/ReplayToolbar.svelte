<script lang="ts">
  import {
    Scissors,
    Play,
    Pause,
    SkipForward,
    SkipBack,
    RotateCcw,
    X,
    Gauge,
    Clock,
    Calendar,
    GripVertical
  } from '@lucide/svelte';
  import type { IReplayEnginePort, ReplayState } from '../../ports/IReplayEnginePort';

  interface Props {
    replayEngine: IReplayEnginePort;
    replayState: ReplayState;
    onOpenGoToDate?: () => void;
  }

  let { replayEngine, replayState, onOpenGoToDate }: Props = $props();

  let isSpeedDropdownOpen = $state(false);

  // Draggable Floating State
  let isDragging = $state(false);
  let dragOffset = { x: 0, y: 0 };
  let toolbarPos = $state<{ x: number; y: number } | null>(null);

  const speedOptions = [
    { label: '0.025s / Bar (40x Hyper)', value: 25 },
    { label: '0.05s / Bar (20x Turbo)', value: 50 },
    { label: '0.1s / Bar (10x Ultra Cepat)', value: 100 },
    { label: '0.2s / Bar (5x Cepat)', value: 200 },
    { label: '0.25s / Bar (4x Cepat)', value: 250 },
    { label: '0.5s / Bar (2x Cepat)', value: 500 },
    { label: '1.0s / Bar (1x Normal)', value: 1000 },
    { label: '2.0s / Bar (0.5x Santai)', value: 2000 },
    { label: '3.0s / Bar (0.3x Teliti)', value: 3000 },
  ];


  function handleSpeedSelect(speedMs: number) {
    replayEngine.setSpeed(speedMs);
    isSpeedDropdownOpen = false;
  }

  function handleSliderChange(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    replayEngine.jumpToIndex(val);
  }

  function handleMouseDown(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('button, input')) return;
    isDragging = true;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dragOffset = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    toolbarPos = {
      x: Math.max(10, Math.min(window.innerWidth - 450, e.clientX - dragOffset.x)),
      y: Math.max(70, Math.min(window.innerHeight - 150, e.clientY - dragOffset.y)),
    };
  }

  function handleMouseUp() {
    isDragging = false;
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }
  function formatReplayTime(timestampSec?: number, isoDate?: string): string {
    let d: Date | null = null;
    if (timestampSec) {
      d = new Date(timestampSec * 1000);
    } else if (isoDate) {
      const parsed = new Date(isoDate);
      if (!isNaN(parsed.getTime())) d = parsed;
    }

    if (!d) return 'Waktu Bar';

    return new Intl.DateTimeFormat(undefined, {
      day: '2-digit',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
      timeZoneName: 'short',
    }).format(d);
  }
</script>

<!-- Floating Glassmorphism TradingView Replay Toolbar -->
<div
  role="toolbar"
  tabindex="0"
  aria-label="Bar Replay Controls"
  onmousedown={handleMouseDown}
  style={toolbarPos ? `position: fixed; left: ${toolbarPos.x}px; top: ${toolbarPos.y}px; z-index: 50;` : ''}
  class="flex flex-col gap-1.5 p-2 bg-[#131722]/95 backdrop-blur-md border border-[#2a2e39] rounded-xl shadow-2xl text-xs font-mono text-[#d1d4dc] select-none transition-shadow {isDragging ? 'cursor-grabbing shadow-[#2962ff]/20' : ''}"
>
  <!-- Top Row: Main Status & Actions -->
  <div class="flex flex-wrap items-center justify-between gap-2">
    <!-- Left: Drag Handle, Status & Cut Tool -->
    <div class="flex items-center gap-1.5">
      <!-- Drag Grip Handle -->
      <div class="p-1 cursor-grab active:cursor-grabbing text-[#787b86] hover:text-white transition-colors" title="Geser panel ke mana saja di layar">
        <GripVertical class="w-3.5 h-3.5" />
      </div>

      <!-- Cut / Select Point Button -->
      <button
        onclick={() => replayEngine.setSelectingCutPoint(!replayState.isSelectingCutPoint)}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-bold transition-all {replayState.isSelectingCutPoint ? 'bg-[#2962ff] text-white shadow-lg ring-2 ring-[#2962ff]/50 animate-pulse' : 'bg-[#1e222d] hover:bg-[#2a2e39] text-[#d1d4dc] border border-[#2a2e39]'}"
        title="Klik untuk memilih titik potong waktu di chart (Garis Biru Vertikal)"
      >
        <Scissors class="w-3.5 h-3.5" />
        <span>{replayState.isSelectingCutPoint ? 'Pilih Bar...' : 'Cut Bar'}</span>
      </button>

      <!-- Go to Date Button (Alt + G) -->
      {#if onOpenGoToDate}
        <button
          onclick={onOpenGoToDate}
          class="flex items-center gap-1 px-2 py-1 rounded-lg bg-[#1e222d] hover:bg-[#2a2e39] border border-[#2a2e39] text-[#787b86] hover:text-white font-bold transition-all"
          title="Lompat ke tanggal spesifik (Shortcut: Alt + G)"
        >
          <Calendar class="w-3.5 h-3.5 text-[#2962ff]" />
          <span>Jump to Date</span>
        </button>
      {/if}

      <!-- Status Badge -->
      {#if replayState.isSelectingCutPoint}
        <span class="text-[11px] font-bold text-[#f5c344] bg-[#f5c344]/15 px-2 py-0.5 rounded border border-[#f5c344]/30 animate-pulse">
          ✂️ Arahkan mouse dan klik candle pada grafik untuk memotong masa lalu
        </span>
      {:else if replayState.isActive}
        <div class="flex items-center gap-1.5 bg-[#f23645]/15 border border-[#f23645]/40 px-2 py-0.5 rounded text-[11px] font-bold text-[#f23645]">
          <span class="w-2 h-2 rounded-full bg-[#f23645] animate-ping"></span>
          <span>REPLAY: Bar {replayState.currentIndex + 1}/{replayState.totalBars}</span>
        </div>
      {/if}
    </div>

    <!-- Center: Playback Controls -->
    {#if replayState.isActive}
      <div class="flex items-center gap-1 bg-[#1e222d] p-1 rounded-lg border border-[#2a2e39]">
        <!-- Jump to Start of Cut -->
        <button
          onclick={() => replayEngine.jumpToIndex(replayState.startIndex)}
          class="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-white transition-colors"
          title="Kembali ke titik awal pemotongan"
        >
          <RotateCcw class="w-3.5 h-3.5" />
        </button>

        <!-- Step Backward 1 Bar (Shift + ←) -->
        <button
          onclick={() => replayEngine.stepBackward()}
          class="p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-white transition-colors"
          title="Mundur 1 Bar (Shortcut: Shift + ←)"
        >
          <SkipBack class="w-3.5 h-3.5" />
        </button>

        <!-- Play / Pause (Space) -->
        {#if replayState.isPlaying}
          <button
            onclick={() => replayEngine.pause()}
            class="flex items-center gap-1 px-3 py-1 bg-[#f5c344] hover:bg-[#e0b138] text-black font-extrabold rounded shadow transition-all"
            title="Jeda Playback (Shortcut: Space)"
          >
            <Pause class="w-3.5 h-3.5 fill-black" />
            <span>Pause</span>
          </button>
        {:else}
          <button
            onclick={() => replayEngine.play()}
            class="flex items-center gap-1 px-3 py-1 bg-[#089981] hover:bg-[#067a67] text-white font-extrabold rounded shadow transition-all"
            title="Mulai Playback Otomatis (Shortcut: Space)"
          >
            <Play class="w-3.5 h-3.5 fill-white" />
            <span>Play</span>
          </button>
        {/if}

        <!-- Step Forward 1 Bar (Shift + →) -->
        <button
          onclick={() => replayEngine.stepForward()}
          class="flex items-center gap-1 px-2.5 py-1 bg-[#2962ff]/20 hover:bg-[#2962ff]/30 text-[#2962ff] hover:text-[#5282ff] border border-[#2962ff]/40 font-bold rounded transition-all"
          title="Maju 1 Bar untuk memeriksa sinyal (Shortcut: Shift + →)"
        >
          <span>Step</span>
          <SkipForward class="w-3.5 h-3.5" />
        </button>
      </div>

      <!-- Right: Speed Selector & Exit Replay -->
      <div class="flex items-center gap-1.5 relative">
        <!-- Speed Selector Button -->
        <div class="relative">
          <button
            onclick={() => isSpeedDropdownOpen = !isSpeedDropdownOpen}
            class="flex items-center gap-1 px-2 py-1 rounded bg-[#1e222d] hover:bg-[#2a2e39] border border-[#2a2e39] text-[#787b86] hover:text-white transition-colors"
            title="Kecepatan Playback"
          >
            <Gauge class="w-3.5 h-3.5 text-[#2962ff]" />
            <span>{(1000 / replayState.speedMs).toFixed(1)}x</span>
          </button>

          <!-- Speed Dropdown Menu -->
          {#if isSpeedDropdownOpen}
            <div
              class="fixed inset-0 z-40"
              onclick={() => isSpeedDropdownOpen = false}
              role="presentation"
            ></div>
            <div class="absolute right-0 bottom-full mb-1.5 w-48 bg-[#1e222d] border border-[#2a2e39] rounded-xl shadow-2xl z-50 overflow-hidden py-1">
              <div class="px-2.5 py-1 text-[10px] text-[#787b86] border-b border-[#2a2e39] font-bold">
                KECEPATAN PLAYBACK
              </div>
              {#each speedOptions as opt}
                {@const isSelected = replayState.speedMs === opt.value}
                <button
                  onclick={() => handleSpeedSelect(opt.value)}
                  class="w-full text-left px-2.5 py-1.5 text-xs flex items-center justify-between transition-colors {isSelected ? 'bg-[#2962ff] text-white font-bold' : 'hover:bg-[#131722] text-[#d1d4dc]'}"
                >
                  <span>{opt.label}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Exit Replay Mode (Jump to Realtime) -->
        <button
          onclick={() => replayEngine.stopReplay()}
          class="flex items-center gap-1 px-2 py-1 rounded bg-[#1e222d] hover:bg-[#f23645]/20 text-[#787b86] hover:text-[#f23645] border border-[#2a2e39] hover:border-[#f23645]/50 transition-all font-bold"
          title="Keluar dari Replay (Kembali ke Grafik Realtime)"
        >
          <X class="w-3.5 h-3.5" />
          <span>Realtime</span>
        </button>
      </div>
    {/if}
  </div>

  <!-- Bottom Row: Timeline Progress Scrubber & Timestamp Info -->
  {#if replayState.isActive}
    <div class="flex items-center gap-2 pt-1 border-t border-[#2a2e39]/60 text-[10px] text-[#787b86]">
      <div class="flex items-center gap-1 text-[#2962ff] bg-[#2962ff]/10 px-2 py-0.5 rounded border border-[#2962ff]/20 flex-shrink-0 font-bold">
        <Clock class="w-3 h-3 text-[#2962ff]" />
        <span>{formatReplayTime(replayState.currentTimestamp, replayState.currentIsoDate)}</span>
      </div>

      <!-- Scrubber Slider -->
      <input
        type="range"
        min={replayState.startIndex}
        max={Math.max(replayState.startIndex, replayState.totalBars - 1)}
        value={replayState.currentIndex}
        oninput={handleSliderChange}
        class="flex-1 h-1.5 bg-[#2a2e39] rounded-lg appearance-none cursor-pointer accent-[#2962ff]"
      />

      <span class="font-mono flex-shrink-0 bg-[#1e222d] px-2 py-0.5 rounded border border-[#2a2e39] text-[#787b86]">
        +{replayState.currentIndex - replayState.startIndex} Bar Diputar
      </span>
    </div>
  {/if}
</div>
