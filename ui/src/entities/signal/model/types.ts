export interface SignalEntity {
  id: string;
  symbol: string;
  action: 'BUY' | 'SELL' | 'BUY_LIMIT' | 'SELL_LIMIT';
  timeframe: string;
  entryPrice: number;
  stopLoss: number;
  takeProfit1: number;
  takeProfit2?: number;
  riskRewardRatio: number;
  confidenceScore: number;
  strategyName: string;
  rationale: string;
  status: 'ACTIVE' | 'TARGET_HIT' | 'SL_HIT' | 'PENDING';
  createdAt: string;
}

export function formatSignalPost(signal: SignalEntity): string {
  const emoji = signal.action.includes('BUY') ? '🟢 BUY' : '🔴 SELL';
  let text = `📊 FOREX SIGNAL ALERT\n━━━━━━━━━━━━━━━━━━\nPair: ${signal.symbol}\nAction: ${emoji}\nEntry: ${signal.entryPrice}\nStop Loss: ${signal.stopLoss}\nTP 1: ${signal.takeProfit1}\n`;
  if (signal.takeProfit2) {
    text += `TP 2: ${signal.takeProfit2}\n`;
  }
  text += `R:R Ratio: 1:${signal.riskRewardRatio.toFixed(2)}\nStrategy: ${signal.strategyName}\nNote: ${signal.rationale}\n━━━━━━━━━━━━━━━━━━`;
  return text;
}
