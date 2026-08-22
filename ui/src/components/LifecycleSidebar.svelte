<script lang="ts">
  import {
    LayoutDashboard,
    GitPullRequest,
    DollarSign,
    LineChart,
    Stethoscope,
    Dices,
    Cpu,
    Zap,
    ChevronLeft,
    ChevronRight,
    ShieldCheck,
    Radio,
    Activity
  } from '@lucide/svelte';

  interface Props {
    activeNav: string;
    onNavClick: (nav: string) => void;
  }

  let { activeNav = 'terminal', onNavClick }: Props = $props();

  let isExpanded = $state(false);

  const navItems = [
    { id: 'terminal', label: 'TradingView Chart', icon: LayoutDashboard, tag: 'H1' },
    { id: 'monte-carlo', label: 'Monte Carlo Lab', icon: Dices, tag: '1K' },
    { id: 'multi-strategy', label: 'Multi-Strategy Matrix', icon: Cpu, tag: 'HUB' },
    { id: 'lifecycle', label: '6-Stage Provenance', icon: GitPullRequest, tag: 'AUDIT' },
    { id: 'tf-hub', label: 'TF Monetization HUD', icon: DollarSign, tag: 'VP' },
    { id: 'wfa-lab', label: 'Walk-Forward WFA Lab', icon: LineChart, tag: 'WFER' },
    { id: 'eda', label: 'EDA Data Health Audit', icon: Stethoscope, tag: 'TICK' },
  ];
</script>

<!-- TradingView-Standard Sleek Left Dock (Slim Icon Rail) -->
<aside
  class="bg-[#1e222d] border border-[#2a2e39] rounded-xl flex flex-col justify-between shadow-lg transition-all duration-300 select-none z-20 {isExpanded ? 'w-64 p-3' : 'w-16 p-2'} shrink-0 font-sans"
>
  <div class="flex flex-col gap-3">
    <!-- Brand Header & Collapse Toggle -->
    <div class="flex items-center {isExpanded ? 'justify-between pb-2 border-b border-[#2a2e39]' : 'justify-center pb-2 border-b border-[#2a2e39]'}">
      <div class="flex items-center gap-2.5">
        <div class="p-2 rounded-xl bg-gradient-to-br from-[#2962ff] to-[#1e4bd8] text-white shadow-md shadow-[#2962ff]/20">
          <Zap class="w-4 h-4 fill-current" />
        </div>
        {#if isExpanded}
          <div class="overflow-hidden whitespace-nowrap">
            <div class="text-xs font-black font-mono tracking-wider text-[#d1d4dc]">HEXAGON QUANT</div>
            <div class="text-[9px] font-mono text-[#2962ff] font-bold tracking-tight">TRADINGVIEW 2026</div>
          </div>
        {/if}
      </div>

      {#if isExpanded}
        <button
          onclick={() => isExpanded = false}
          class="p-1 rounded-lg text-[#787b86] hover:text-white hover:bg-[#2a2e39] transition-colors"
          title="Collapse Sidebar"
        >
          <ChevronLeft class="w-4 h-4" />
        </button>
      {/if}
    </div>

    <!-- Navigation Buttons -->
    <nav class="flex flex-col gap-1.5">
      {#each navItems as item}
        {@const IconComponent = item.icon}
        {@const isActive = activeNav === item.id}
        <div class="relative group">
          <button
            onclick={() => onNavClick(item.id)}
            class="w-full flex items-center {isExpanded ? 'gap-3 px-3 py-2' : 'justify-center p-2.5'} rounded-xl text-xs font-semibold font-mono transition-all {isActive ? 'bg-[#2962ff] text-white shadow-md shadow-[#2962ff]/30 font-bold' : 'text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39]'}"
          >
            <IconComponent class="w-4 h-4 shrink-0 {isActive ? 'text-white' : 'text-[#787b86] group-hover:text-white'}" />

            {#if isExpanded}
              <span class="truncate flex-1 text-left">{item.label}</span>
              <span class="text-[9px] px-1.5 py-0.5 rounded font-mono font-bold {isActive ? 'bg-white/20 text-white' : 'bg-[#131722] text-[#787b86]'}">
                {item.tag}
              </span>
            {/if}
          </button>

          <!-- Floating Tooltip when collapsed -->
          {#if !isExpanded}
            <div class="fixed left-20 z-50 pointer-events-none hidden group-hover:flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-[#1e222d] border border-[#2a2e39] text-[#d1d4dc] text-xs font-mono font-bold shadow-2xl whitespace-nowrap animate-in fade-in zoom-in-95 duration-150">
              <span>{item.label}</span>
              <span class="text-[9px] px-1.5 py-0.2 rounded bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/40">
                {item.tag}
              </span>
            </div>
          {/if}
        </div>
      {/each}
    </nav>
  </div>

  <!-- Bottom Telemetry Status -->
  <div class="flex flex-col gap-2 pt-2 border-t border-[#2a2e39]">
    {#if !isExpanded}
      <button
        onclick={() => isExpanded = true}
        class="w-full flex items-center justify-center p-2 rounded-lg text-[#787b86] hover:text-white hover:bg-[#2a2e39] transition-colors"
        title="Expand Sidebar"
      >
        <ChevronRight class="w-4 h-4" />
      </button>

      <!-- Mini Status Dot Indicators with Tooltip -->
      <div class="flex flex-col items-center gap-1.5 py-1" title="System Telemetry: IPC 0.4ms • 100% Dukascopy • 0-Penalty">
        <span class="w-2 h-2 rounded-full bg-[#089981] animate-pulse" title="MT5 IPC: 0.4ms"></span>
        <span class="w-2 h-2 rounded-full bg-[#2962ff]" title="Dukascopy Feed: Real"></span>
        <span class="w-2 h-2 rounded-full bg-[#089981]" title="Compliance: 0-Penalty"></span>
      </div>
    {:else}
      <!-- Expanded Full Telemetry Card -->
      <div class="bg-[#131722] p-2.5 rounded-xl border border-[#2a2e39] text-[10px] font-mono space-y-1">
        <div class="flex items-center justify-between text-[#787b86]">
          <span class="flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-[#089981] animate-ping"></span> MT5 Bridge IPC:
          </span>
          <span class="text-[#d1d4dc] font-bold">0.4ms</span>
        </div>
        <div class="flex items-center justify-between text-[#787b86]">
          <span class="flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-[#2962ff]"></span> Dukascopy Feed:
          </span>
          <span class="text-[#2962ff] font-bold">100% Real</span>
        </div>
        <div class="flex items-center justify-between text-[#787b86]">
          <span class="flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-[#089981]"></span> TF Compliance:
          </span>
          <span class="text-[#089981] font-bold">0-Penalty</span>
        </div>
      </div>
    {/if}
  </div>
</aside>
