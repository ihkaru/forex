<script lang="ts">
  import {
    X,
    Database,
    Activity,
    Award,
    TrendingUp,
    Stethoscope,
    Dices,
    Cpu
  } from '@lucide/svelte';
  import type { EdaReport, Candle } from '../domain/models';
  import type { MonteCarloReport, TfScorecardReport, StrategyDescriptor } from '../ports';

  // Sub-modal specialized components (Single Responsibility Principle)
  import DataProvenanceModal from './modals/DataProvenanceModal.svelte';
  import LifecycleModal from './modals/LifecycleModal.svelte';
  import TfScorecardModal from './modals/TfScorecardModal.svelte';
  import MonteCarloModal from './modals/MonteCarloModal.svelte';
  import EdaHealthModal from './modals/EdaHealthModal.svelte';
  import ModelHubModal from './modals/ModelHubModal.svelte';

  interface Props {
    isOpen: boolean;
    modalType: 'data-provenance' | 'lifecycle' | 'tf-hub' | 'wfa-lab' | 'eda' | 'monte-carlo' | 'multi-strategy';
    activeSymbol?: string;
    edaReport?: EdaReport | null;
    candles?: Candle[];
    monteCarloData?: MonteCarloReport | null;
    scorecardData?: TfScorecardReport | null;
    strategies?: StrategyDescriptor[];
    selectedStrategyId?: string;
    onSelectStrategy?: (strategyId: string) => void;
    onClose: () => void;
  }

  let {
    isOpen,
    modalType,
    activeSymbol = 'XAUUSD',
    edaReport = null,
    candles = [],
    monteCarloData = null,
    scorecardData = null,
    strategies = [],
    selectedStrategyId = 'pola-n-core',
    onSelectStrategy,
    onClose
  }: Props = $props();
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-150">
    <!-- Modal Container -->
    <div
      class="bg-[#1e222d] border border-[#2a2e39] rounded-2xl w-full {modalType === 'multi-strategy' ? 'max-w-4xl' : 'max-w-3xl'} overflow-hidden shadow-2xl flex flex-col max-h-[90vh] animate-in zoom-in-95 duration-150"
    >
      <!-- Modal Header -->
      <div class="p-4 px-6 border-b border-[#2a2e39] flex items-center justify-between bg-[#131722]/90 sticky top-0 z-10 backdrop-blur">
        <div class="flex items-center gap-2.5">
          {#if modalType === 'data-provenance'}
            <Database class="w-5 h-5 text-[#089981]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Market Data Provenance &amp; Lineage Inspector • {activeSymbol}</h2>
          {:else if modalType === 'lifecycle'}
            <Activity class="w-5 h-5 text-[#2962ff]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">6-Stage Quantitative Signal Provenance Lifecycle</h2>
          {:else if modalType === 'tf-hub'}
            <Award class="w-5 h-5 text-[#f5c344]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Traders Family Monetization Engine &amp; 7-Pillar Scorecard</h2>
          {:else if modalType === 'wfa-lab'}
            <TrendingUp class="w-5 h-5 text-[#089981]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Walk-Forward Analysis (WFA) Anti-Overfitting Lab</h2>
          {:else if modalType === 'eda'}
            <Stethoscope class="w-5 h-5 text-[#2962ff]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Exploratory Data Analysis (EDA) • {activeSymbol}</h2>
          {:else if modalType === 'monte-carlo'}
            <Dices class="w-5 h-5 text-[#ab47bc]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Monte Carlo 1,000-Path Equity Simulation &amp; Risk-of-Ruin • {activeSymbol}</h2>
          {:else if modalType === 'multi-strategy'}
            <Cpu class="w-5 h-5 text-[#2962ff]" />
            <h2 class="text-base font-bold text-[#d1d4dc]">Multi-Strategy Quantitative Engine Matrix</h2>
          {/if}
        </div>

        <button
          onclick={onClose}
          aria-label="Tutup Dialog"
          class="p-1.5 rounded-lg bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#f23645]/20 hover:text-[#f23645] transition-all"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Modal Body (SRP Modular Routing) -->
      <div class="p-6 text-xs text-[#d1d4dc] overflow-y-auto">
        {#if modalType === 'data-provenance'}
          <DataProvenanceModal {activeSymbol} {edaReport} {candles} />
        {:else if modalType === 'monte-carlo'}
          <MonteCarloModal {activeSymbol} {monteCarloData} />
        {:else if modalType === 'multi-strategy'}
          <ModelHubModal
            {strategies}
            {selectedStrategyId}
            {onSelectStrategy}
            {onClose}
          />
        {:else if modalType === 'lifecycle'}
          <LifecycleModal />
        {:else if modalType === 'tf-hub'}
          <TfScorecardModal {scorecardData} />
        {:else if modalType === 'eda'}
          <EdaHealthModal {edaReport} />
        {/if}
      </div>
    </div>
  </div>
{/if}
