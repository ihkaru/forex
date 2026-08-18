import type { DetailedBacktestReport, ITesterPort } from '../../ports/ITesterPort';

export class RestTesterAdapter implements ITesterPort {
  constructor(private readonly baseUrl: string = 'http://127.0.0.1:5000/api') {}

  async getDetailedBacktestReport(symbol: string): Promise<DetailedBacktestReport> {
    const cleanSym = symbol.replace(/[^A-Za-z]/g, '').toUpperCase();
    const url = `${this.baseUrl}/backtest/detailed/${cleanSym}`;
    
    const resp = await fetch(url);
    if (!resp.ok) {
      throw new Error(`[RestTesterAdapter] HTTP ${resp.status}: Gagal memuat detailed backtest`);
    }

    const data = await resp.json();
    const rep = data.report;
    const summary = rep.summary || {
      all: {
        total_trades: rep.total_trades || 0,
        winning_trades: rep.winning_trades || 0,
        losing_trades: rep.losing_trades || 0,
        win_rate_pct: rep.win_rate_percent || 0,
        gross_profit_pips: rep.gross_profit_pips || 0,
        gross_loss_pips: rep.gross_loss_pips || 0,
        net_pips: rep.total_raw_pips || 0,
        profit_factor: rep.profit_factor || 0,
      },
      long: {
        total_trades: 0,
        winning_trades: 0,
        losing_trades: 0,
        win_rate_pct: 0,
        gross_profit_pips: 0,
        gross_loss_pips: 0,
        net_pips: 0,
        profit_factor: 0,
      },
      short: {
        total_trades: 0,
        winning_trades: 0,
        losing_trades: 0,
        win_rate_pct: 0,
        gross_profit_pips: 0,
        gross_loss_pips: 0,
        net_pips: 0,
        profit_factor: 0,
      },
      largest_win_pips: 0,
      largest_loss_pips: 0,
      max_consecutive_wins: 0,
      max_consecutive_losses: 0,
      avg_trade_pips: 0,
      avg_win_pips: 0,
      avg_loss_pips: 0,
      payoff_ratio: 0,
      avg_bars_held: 0,
      max_drawdown_pips: rep.max_drawdown_pips || 0,
      max_drawdown_pct: 0,
      sharpe_ratio: 0,
      sortino_ratio: 0,
    };

    return {
      symbol: cleanSym,
      timeframe: rep.timeframe || 'H1',
      total_trades: rep.total_trades || 0,
      winning_trades: rep.winning_trades || 0,
      losing_trades: rep.losing_trades || 0,
      win_rate_percent: rep.win_rate_percent || 0,
      total_raw_pips: rep.total_raw_pips || 0,
      total_valued_pips: rep.total_valued_pips || 0,
      gross_profit_pips: rep.gross_profit_pips || 0,
      gross_loss_pips: rep.gross_loss_pips || 0,
      profit_factor: rep.profit_factor || 0,
      max_drawdown_pips: rep.max_drawdown_pips || 0,
      recovery_factor: rep.recovery_factor || 0,
      monthly_loss_ratio_percent: rep.monthly_loss_ratio_percent || 0,
      is_tf_qualified: rep.is_tf_qualified || false,
      summary,
      trades: data.trades || [],
      equity_curve: data.equity_curve || [],
    };
  }
}
