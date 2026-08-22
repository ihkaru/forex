export interface TradeDirectionBreakdown {
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate_pct: number;
  gross_profit_pips: number;
  gross_loss_pips: number;
  net_pips: number;
  profit_factor: number;
}

export interface TradingViewPerformanceSummary {
  all: TradeDirectionBreakdown;
  long: TradeDirectionBreakdown;
  short: TradeDirectionBreakdown;
  largest_win_pips: number;
  largest_loss_pips: number;
  max_consecutive_wins: number;
  max_consecutive_losses: number;
  avg_trade_pips: number;
  avg_win_pips: number;
  avg_loss_pips: number;
  payoff_ratio: number;
  avg_bars_held: number;
  max_drawdown_pips: number;
  max_drawdown_pct: number;
  sharpe_ratio: number;
  sortino_ratio: number;
}

export interface EquityCurvePoint {
  time: number;
  equity_pips: number;
  drawdown_pips: number;
  drawdown_percent: number;
}

export interface DetailedBacktestReport {
  symbol: string;
  timeframe: string;
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate_percent: number;
  total_raw_pips: number;
  total_valued_pips: number;
  gross_profit_pips: number;
  gross_loss_pips: number;
  profit_factor: number;
  max_drawdown_pips: number;
  recovery_factor: number;
  monthly_loss_ratio_percent: number;
  is_tf_qualified: boolean;
  summary: TradingViewPerformanceSummary;
  trades: Array<{
    id: string;
    symbol: string;
    action: string;
    open_time: number;
    open_price: number;
    close_time: number;
    close_price: number;
    stop_loss: number;
    take_profit: number;
    pnl_pips: number;
    valued_pips: number;
    is_win: boolean;
    exit_reason: string;
  }>;
  equity_curve: EquityCurvePoint[];
}

export interface ITesterPort {
  getDetailedBacktestReport(symbol: string, strategyId?: string): Promise<DetailedBacktestReport>;
}
