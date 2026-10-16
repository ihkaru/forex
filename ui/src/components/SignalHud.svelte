<script lang="ts">
  import type {
    Signal,
    ExecutionHudStatus,
    RunningPositionState,
    SettledTradeState,
    MarketScanContext,
    PendingOrderState
  } from '../domain/models';
  import type { PointInTimeComplianceState } from '../ports';
  import RunningPositionCard from './hud/RunningPositionCard.svelte';
  import SettledTradeCard from './hud/SettledTradeCard.svelte';
  import PendingOrderCard from './hud/PendingOrderCard.svelte';
  import MarketRadarCard from './hud/MarketRadarCard.svelte';
  import BentoKpiSummary from './hud/BentoKpiSummary.svelte';

  interface Props {
    status?: ExecutionHudStatus;
    signal?: Signal | null;
    pendingOrder?: PendingOrderState | null;
    activeSymbol?: string;
    activeStrategyName?: string;
    runningPosition?: RunningPositionState | null;
    settledTrade?: SettledTradeState | null;
    scanContext?: MarketScanContext | null;
    valuedPips?: number;
    currentMonthLabel?: string;
    currentMonthVp?: number;
    currentMonthTrades?: number;
    targetPips?: number;
    scorecardScore?: number;
    scorecardTier?: string;
    scorecardPillars?: any[];
    wferPct?: number;
    totalBars?: number;
    complianceState?: PointInTimeComplianceState | null;
    onScanSignal?: () => void;
  }

  let {
    status = 'SCANNING',
    signal = null,
    pendingOrder = null,
    activeSymbol = 'XAUUSD',
    activeStrategyName = 'TF Pola N Adaptive (v2 Gold Specialist)',
    runningPosition = null,
    settledTrade = null,
    scanContext = null,
    valuedPips = 3262.5,
    currentMonthLabel = 'Monthly Goal',
    currentMonthVp = 0.0,
    currentMonthTrades = 0,
    targetPips = 300.0,
    scorecardScore = 20,
    scorecardTier = 'MASTER_PRIORITY',
    scorecardPillars = [],
    wferPct = 94.8,
    totalBars = 198534,
    complianceState = null,
    onScanSignal
  }: Props = $props();

  // Dynamic Effective State
  let effectiveStatus = $derived.by<ExecutionHudStatus>(() => {
    if (runningPosition) return 'RUNNING';
    if (settledTrade) return 'SETTLED';
    if (pendingOrder) return 'PENDING';
    if (signal) return 'PENDING';
    return status;
  });
</script>

<div class="flex flex-col gap-3 font-sans select-none">
  <!-- Dynamic State Cards -->
  {#if effectiveStatus === 'RUNNING' && runningPosition}
    <RunningPositionCard {runningPosition} />
  {:else if effectiveStatus === 'SETTLED' && settledTrade}
    <SettledTradeCard {settledTrade} />
  {:else if effectiveStatus === 'PENDING'}
    <PendingOrderCard
      {pendingOrder}
      {signal}
      {activeSymbol}
      {activeStrategyName}
      {onScanSignal}
    />
  {:else}
    <MarketRadarCard
      {activeSymbol}
      {scanContext}
      {complianceState}
      {onScanSignal}
    />
  {/if}

  <!-- Bento Intelligence Summary -->
  <BentoKpiSummary
    {currentMonthLabel}
    {currentMonthVp}
    {currentMonthTrades}
    {targetPips}
    {valuedPips}
    {scorecardScore}
    {scorecardTier}
    {scorecardPillars}
    {wferPct}
    {totalBars}
  />
</div>
