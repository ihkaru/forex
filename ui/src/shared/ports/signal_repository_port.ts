export interface SignalDto {
  id: string;
  symbol: string;
  action: 'BUY' | 'SELL' | 'BUY_LIMIT' | 'SELL_LIMIT';
  timeframe: string;
  entryPrice: number;
  stopLoss: number;
  takeProfit1: number;
  takeProfit2?: number;
  takeProfit3?: number;
  riskRewardRatio: number;
  confidenceScore: number;
  strategyName: string;
  rationale: string;
  status: 'ACTIVE' | 'TARGET_HIT' | 'SL_HIT' | 'PENDING';
  createdAt: string;
}

export interface SignalRepositoryPort {
  getActiveSignals(): Promise<SignalDto[]>;
  getSignalHistory(limit: number): Promise<SignalDto[]>;
}
