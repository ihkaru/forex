<script lang="ts">
  import {
    LayoutDashboard,
    GitPullRequest,
    DollarSign,
    LineChart,
    Stethoscope,
    Database,
    Activity,
    GitBranch,
    TrendingUp,
    ShieldCheck,
    Radio,
    Dices,
    Cpu,
    Zap
  } from '@lucide/svelte';

  interface Props {
    activeNav: string;
    onNavClick: (nav: string) => void;
  }

  let { activeNav = 'terminal', onNavClick }: Props = $props();

  const navItems = [
    { id: 'terminal', label: 'TradingView Chart', icon: LayoutDashboard },
    { id: 'monte-carlo', label: 'Monte Carlo Lab (1K)', icon: Dices },
    { id: 'multi-strategy', label: 'Multi-Strategy Matrix', icon: Cpu },
    { id: 'lifecycle', label: '6-Stage Provenance', icon: GitPullRequest },
    { id: 'tf-hub', label: 'TF Monetization HUD', icon: DollarSign },
    { id: 'wfa-lab', label: 'Walk-Forward WFA Lab', icon: LineChart },
    { id: 'eda', label: 'EDA Data Health Audit', icon: Stethoscope },
  ];

  const pipelineStages = [
    { id: 1, title: '1. Market Ingestion', desc: '100% Dukascopy Real Swiss TickFeed (UTC Norm)', icon: Database },
    { id: 2, title: '2. EDA Health Check', desc: '0 Math Violation • Max Spread < 5.0 Pips', icon: Activity },
    { id: 3, title: '3. Multi-Strategy Core', desc: 'Pola N • Dual EMA • ICT Order Block', icon: GitBranch },
    { id: 4, title: '4. Anti-Bias WFA', desc: 'WFER 94.8% • Zero Look-Ahead Rolling Matrix', icon: TrendingUp },
    { id: 5, title: '5. TF Compliance Guard', desc: 'Pending Limit Only • RR <= 1:3.00 • 0-Penalty', icon: ShieldCheck },
    { id: 6, title: '6. Priority Broadcast', desc: 'Auto-Dispatched to Traders Family VIP', icon: Radio },
  ];
</script>

<aside class="w-full lg:w-72 bg-[#1e222d] border border-[#2a2e39] rounded-xl p-4 flex flex-col justify-between shadow-md gap-5 font-sans">
  <div class="flex flex-col gap-4">
    <!-- Brand Logo Header (TradingView Style) -->
    <div class="flex items-center gap-3 pb-3 border-b border-[#2a2e39]">
      <div class="p-2 rounded-xl bg-[#2962ff] text-white shadow-md">
        <Zap class="w-5 h-5 fill-current" />
      </div>
      <div>
        <div class="text-sm font-black font-mono tracking-wider text-[#d1d4dc]">HEXAGON QUANT</div>
        <div class="text-[10px] font-mono text-[#2962ff] font-semibold tracking-tight">TRADINGVIEW ENGINE • 2026</div>
      </div>
    </div>

    <!-- Navigation Menu -->
    <nav class="flex flex-col gap-1">
      {#each navItems as item}
        {@const IconComponent = item.icon}
        <button
          onclick={() => onNavClick(item.id)}
          class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-semibold font-mono transition-all text-left {activeNav === item.id ? 'bg-[#2962ff]/20 text-[#2962ff] border border-[#2962ff]/40 shadow-sm' : 'text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39] border border-transparent'}"
        >
          <IconComponent class="w-4 h-4 {activeNav === item.id ? 'text-[#2962ff]' : 'text-[#787b86]'}" />
          <span>{item.label}</span>
        </button>
      {/each}
    </nav>

    <!-- 6-Stage Signal Provenance Stepper -->
    <div class="bg-[#131722] p-3 rounded-xl border border-[#2a2e39]">
      <div class="flex items-center justify-between pb-2 mb-2 border-b border-[#2a2e39]">
        <span class="text-[11px] font-bold font-mono text-[#d1d4dc] flex items-center gap-1.5">
          <GitPullRequest class="w-3.5 h-3.5 text-[#2962ff]" /> 6-Stage Provenance
        </span>
        <span class="text-[9px] font-mono px-1.5 py-0.5 rounded bg-[#089981]/20 text-[#089981] font-bold">
          LIVE AUDIT
        </span>
      </div>

      <div class="space-y-2">
        {#each pipelineStages as stage}
          {@const StageIcon = stage.icon}
          <div class="flex items-start gap-2">
            <div class="p-1 rounded bg-[#089981]/10 text-[#089981] border border-[#089981]/30 shrink-0 mt-0.5">
              <StageIcon class="w-3 h-3" />
            </div>
            <div>
              <div class="text-[11px] font-bold font-mono text-[#d1d4dc]">{stage.title}</div>
              <div class="text-[9px] text-[#787b86] leading-tight">{stage.desc}</div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Telemetry Card -->
  <div class="bg-[#131722] p-3 rounded-xl border border-[#2a2e39] text-[11px] font-mono space-y-1.5">
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
        <span class="w-1.5 h-1.5 rounded-full bg-[#089981]"></span> Compliance Guard:
      </span>
      <span class="text-[#089981] font-bold">0-Penalty</span>
    </div>
  </div>
</aside>
