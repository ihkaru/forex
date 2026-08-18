import type { SignalRepositoryPort } from '@shared/ports';
import type { SignalEntity } from '@entities/signal';

export async function executeMarketScan(repo: SignalRepositoryPort): Promise<SignalEntity[]> {
  const dtos = await repo.getActiveSignals();
  return dtos.map(dto => ({
    id: dto.id,
    symbol: dto.symbol,
    action: dto.action,
    timeframe: dto.timeframe,
    entryPrice: dto.entryPrice,
    stopLoss: dto.stopLoss,
    takeProfit1: dto.takeProfit1,
    takeProfit2: dto.takeProfit2,
    riskRewardRatio: dto.riskRewardRatio,
    confidenceScore: dto.confidenceScore,
    strategyName: dto.strategyName,
    rationale: dto.rationale,
    status: dto.status,
    createdAt: dto.createdAt,
  }));
}
