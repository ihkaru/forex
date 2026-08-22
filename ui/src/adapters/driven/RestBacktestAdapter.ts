import type { BacktestReport } from '../../domain/models';
import type { IBacktestPort } from '../../ports';

export class RestBacktestAdapter implements IBacktestPort {
  constructor(private readonly baseUrl: string = 'http://127.0.0.1:5000/api') {}

  async runBacktest(): Promise<{
    reports: BacktestReport[];
    totalValuedPips: number;
    portfolioWinRatePct: number;
    wferPct: number;
    isTfQualified: boolean;
  }> {
    const res = await fetch(`${this.baseUrl}/backtest`);
    if (!res.ok) {
      throw new Error(`[RestBacktestAdapter] HTTP error ${res.status}: ${res.statusText}`);
    }
    const data = await res.json();
    return {
      reports: data.reports.map((r: any) => ({
        symbol: r.symbol.base + r.symbol.quote,
        totalTrades: r.total_trades,
        winningTrades: r.winning_trades,
        losingTrades: r.losing_trades,
        winRatePercent: Number(r.win_rate_percent),
        totalRawPips: Number(r.total_raw_pips),
        totalValuedPips: Number(r.total_valued_pips),
        profitFactor: Number(r.profit_factor),
        recoveryFactor: Number(r.recovery_factor),
        isTfQualified: r.is_tf_qualified,
      })),
      totalValuedPips: Number(data.total_valued_pips),
      portfolioWinRatePct: Number(data.portfolio_win_rate_pct),
      wferPct: Number(data.walk_forward_efficiency_ratio_pct),
      isTfQualified: data.is_tf_qualified,
    };
  }

  async getTrades(symbol: string, strategyId?: string): Promise<any[]> {
    try {
      const query = strategyId ? `?strategy=${strategyId}` : '';
      const res = await fetch(`${this.baseUrl}/backtest/trades/${symbol}${query}`);
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.warn(`[RestBacktestAdapter] Gagal fetch trade log untuk ${symbol}:`, e);
    }
    return [];
  }
}
