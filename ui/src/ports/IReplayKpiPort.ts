/**
 * IReplayKpiPort.ts
 * Interface-First Domain Contract for Point-in-Time KPI & Compliance Calculation.
 * Pure, deterministic, zero I/O.
 */

export interface PointInTimeComplianceState {
  noInstantOrder: boolean;
  rrRangeValid: boolean;
  slaMinutesGuaranteed: boolean;
  activeSignalsOnPair: number;
  maxSignalsPerPair: number;
  isCompliant: boolean;
}

export interface PointInTimeScorecardPillar {
  code: string;
  name: string;
  score: number;
  maxScore: number;
  status: 'MAX' | 'ACCEPTABLE' | 'MODERATE' | 'LOW';
  valueText: string;
}

export interface PointInTimeKpiResult {
  currentMonthLabel: string;
  currentMonthVp: number;
  currentMonthTrades: number;
  targetPips: number;
  targetProgressPct: number;
  rewardCashIdr: string;
  isMonthQualified: boolean;
  allTimeValuedPips: number;
  allTimeTradesCount: number;
  pointInTimeWinRatePct: number;
  pointInTimeProfitFactor: number;
  pointInTimeRecoveryFactor: number;
  scorecardScore: number;
  scorecardMaxScore: number;
  scorecardPct: number;
  scorecardTier: string;
  pillars: PointInTimeScorecardPillar[];
  wferPct: number;
  verifiedBarsCount: number;
  compliance: PointInTimeComplianceState;
}

export interface IReplayKpiPort {
  /**
   * Computes point-in-time metrics (Monthly Rewards, 7-Pillars Scorecard, WFER, and Live Compliance)
   * up to the given current timestamp without any lookahead bias.
   */
  computePointInTimeKpis(
    trades: any[],
    currentTimestamp: number,
    activeSymbol: string,
    candlesCount?: number
  ): PointInTimeKpiResult;
}
