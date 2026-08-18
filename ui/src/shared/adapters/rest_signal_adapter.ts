import type { BroadcastReceipt, TraderFamilyPort } from '../ports/trader_family_port';
import type { SignalDto, SignalRepositoryPort } from '../ports/signal_repository_port';

export class RestSignalAdapter implements SignalRepositoryPort, TraderFamilyPort {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://127.0.0.1:8080/api/v1') {
    this.baseUrl = baseUrl;
  }

  async getActiveSignals(): Promise<SignalDto[]> {
    return [
      {
        id: 'sig-uuid-001',
        symbol: 'EUR/USD',
        action: 'BUY',
        timeframe: 'M15',
        entryPrice: 1.085,
        stopLoss: 1.083,
        takeProfit1: 1.089,
        takeProfit2: 1.092,
        riskRewardRatio: 2.0,
        confidenceScore: 0.92,
        strategyName: 'SMC Asian Liquidity Sweep',
        rationale: 'Sweep pada low sesi Asia + Bullish CHoCH pada timeframe M15.',
        status: 'ACTIVE',
        createdAt: new Date().toISOString(),
      },
      {
        id: 'sig-uuid-002',
        symbol: 'GBP/USD',
        action: 'SELL',
        timeframe: 'H1',
        entryPrice: 1.2785,
        stopLoss: 1.2815,
        takeProfit1: 1.2725,
        takeProfit2: 1.268,
        riskRewardRatio: 2.5,
        confidenceScore: 0.88,
        strategyName: 'ICT Silver Bullet FVG',
        rationale: 'Bearish Fair Value Gap rejection during London NY overlap.',
        status: 'ACTIVE',
        createdAt: new Date().toISOString(),
      },
      {
        id: 'sig-uuid-003',
        symbol: 'XAU/USD',
        action: 'BUY',
        timeframe: 'M15',
        entryPrice: 2385.5,
        stopLoss: 2377.0,
        takeProfit1: 2402.5,
        takeProfit2: 2415.0,
        riskRewardRatio: 2.8,
        confidenceScore: 0.95,
        strategyName: 'Order Block Retest',
        rationale: 'Mitigation of demand zone + Macro US Yield cooling.',
        status: 'ACTIVE',
        createdAt: new Date().toISOString(),
      },
    ];
  }

  async getSignalHistory(limit: number): Promise<SignalDto[]> {
    const active = await this.getActiveSignals();
    return active.slice(0, limit);
  }

  async broadcastSignal(signalId: string): Promise<BroadcastReceipt> {
    return {
      signalId,
      channelId: 'tf_channel_vip_quant',
      postId: `tf-post-${Date.now()}`,
      subscribersCount: 1482,
      publishedAt: new Date().toISOString(),
    };
  }

  async getChannelSubscribersCount(): Promise<number> {
    return 1482;
  }
}
