<script lang="ts">
  import type { TradingViewPerformanceSummary } from '../../ports/ITesterPort';

  let { summary }: { summary: TradingViewPerformanceSummary } = $props();

  function fmtPips(val: number, forcePlus: boolean = false): string {
    const num = Number(val) || 0;
    if (forcePlus && num > 0) return `+${num.toFixed(1)} pips`;
    if (num < 0) return `-${Math.abs(num).toFixed(1)} pips`;
    return `${num.toFixed(1)} pips`;
  }

  function fmtLossPips(val: number): string {
    const num = Math.abs(Number(val) || 0);
    return num > 0 ? `-${num.toFixed(1)} pips` : `0.0 pips`;
  }

  function fmtProfitPips(val: number): string {
    const num = Math.abs(Number(val) || 0);
    return num > 0 ? `+${num.toFixed(1)} pips` : `0.0 pips`;
  }

  function fmtPct(val: number): string {
    return `${(Number(val) || 0).toFixed(1)}%`;
  }

  function fmtNum(val: number): string {
    return (Number(val) || 0).toFixed(2);
  }
</script>

<div class="overflow-x-auto w-full">
  <table class="w-full text-xs text-left border-collapse">
    <thead>
      <tr class="border-b border-[#2a2e39] text-[#787b86] font-medium bg-[#1e222d]">
        <th class="py-2.5 px-4 font-semibold text-[#d1d4dc]">Performance Metric</th>
        <th class="py-2.5 px-4 text-right font-semibold text-[#d1d4dc]">All Trades</th>
        <th class="py-2.5 px-4 text-right font-semibold text-[#2962ff]">Long Trades</th>
        <th class="py-2.5 px-4 text-right font-semibold text-[#e040fb]">Short Trades</th>
      </tr>
    </thead>
    <tbody class="divide-y divide-[#2a2e39]/50 font-mono">
      <!-- Net Profit -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Total Net Profit</td>
        <td class="py-2 px-4 text-right font-bold {summary.all.net_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
          {fmtPips(summary.all.net_pips, true)}
        </td>
        <td class="py-2 px-4 text-right {summary.long.net_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
          {fmtPips(summary.long.net_pips, true)}
        </td>
        <td class="py-2 px-4 text-right {summary.short.net_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'}">
          {fmtPips(summary.short.net_pips, true)}
        </td>
      </tr>

      <!-- Gross Profit -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#787b86] font-sans pl-6">Gross Profit</td>
        <td class="py-2 px-4 text-right text-[#089981]">{fmtProfitPips(summary.all.gross_profit_pips)}</td>
        <td class="py-2 px-4 text-right text-[#089981]">{fmtProfitPips(summary.long.gross_profit_pips)}</td>
        <td class="py-2 px-4 text-right text-[#089981]">{fmtProfitPips(summary.short.gross_profit_pips)}</td>
      </tr>

      <!-- Gross Loss -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#787b86] font-sans pl-6">Gross Loss</td>
        <td class="py-2 px-4 text-right text-[#f23645]">{fmtLossPips(summary.all.gross_loss_pips)}</td>
        <td class="py-2 px-4 text-right text-[#f23645]">{fmtLossPips(summary.long.gross_loss_pips)}</td>
        <td class="py-2 px-4 text-right text-[#f23645]">{fmtLossPips(summary.short.gross_loss_pips)}</td>
      </tr>

      <!-- Profit Factor -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Profit Factor (PF)</td>
        <td class="py-2 px-4 text-right font-bold {summary.all.profit_factor >= 1.5 ? 'text-[#089981]' : 'text-[#f23645]'}">
          {fmtNum(summary.all.profit_factor)}
        </td>
        <td class="py-2 px-4 text-right">{fmtNum(summary.long.profit_factor)}</td>
        <td class="py-2 px-4 text-right">{fmtNum(summary.short.profit_factor)}</td>
      </tr>

      <!-- Total Closed Trades -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Total Closed Trades</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc] font-bold">{summary.all.total_trades}</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">{summary.long.total_trades}</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">{summary.short.total_trades}</td>
      </tr>

      <!-- Percent Profitable -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Percent Profitable (Win Rate)</td>
        <td class="py-2 px-4 text-right font-bold {summary.all.win_rate_pct >= 50 ? 'text-[#089981]' : 'text-[#f23645]'}">
          {fmtPct(summary.all.win_rate_pct)}
        </td>
        <td class="py-2 px-4 text-right">{fmtPct(summary.long.win_rate_pct)}</td>
        <td class="py-2 px-4 text-right">{fmtPct(summary.short.win_rate_pct)}</td>
      </tr>

      <!-- Winning / Losing Trades -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#787b86] font-sans pl-6">Winning Trades / Losing Trades</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">{summary.all.winning_trades} / {summary.all.losing_trades}</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">{summary.long.winning_trades} / {summary.long.losing_trades}</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">{summary.short.winning_trades} / {summary.short.losing_trades}</td>
      </tr>

      <!-- Avg Trade Return -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Avg Trade Return</td>
        <td class="py-2 px-4 text-right {summary.avg_trade_pips >= 0 ? 'text-[#089981]' : 'text-[#f23645]'} font-bold">
          {fmtPips(summary.avg_trade_pips, true)}
        </td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
      </tr>

      <!-- Avg Winning Trade vs Avg Losing Trade -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#787b86] font-sans pl-6">Avg Winning Trade / Avg Losing Trade</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">
          <span class="text-[#089981]">{fmtProfitPips(summary.avg_win_pips)}</span> / <span class="text-[#f23645]">{fmtLossPips(summary.avg_loss_pips)}</span>
        </td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
      </tr>

      <!-- Payoff Ratio -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#787b86] font-sans pl-6">Win/Loss Payoff Ratio</td>
        <td class="py-2 px-4 text-right font-bold text-[#2962ff]">{fmtNum(summary.payoff_ratio)}</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
      </tr>

      <!-- Largest Winning Trade & Largest Losing Trade -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Largest Winning Trade / Largest Losing Trade</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">
          <span class="text-[#089981]">{fmtProfitPips(summary.largest_win_pips)}</span> / <span class="text-[#f23645]">{fmtLossPips(summary.largest_loss_pips)}</span>
        </td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
      </tr>

      <!-- Max Consecutive Wins / Losses -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Max Consecutive Wins / Losses</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">
          <span class="text-[#089981] font-bold">{summary.max_consecutive_wins}</span> / <span class="text-[#f23645] font-bold">{summary.max_consecutive_losses}</span>
        </td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
      </tr>

      <!-- Max Equity Drawdown -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Max Equity Drawdown</td>
        <td class="py-2 px-4 text-right text-[#f23645] font-bold">
          {fmtLossPips(summary.max_drawdown_pips)} ({fmtPct(summary.max_drawdown_pct)})
        </td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
      </tr>

      <!-- Sharpe & Sortino -->
      <tr class="hover:bg-[#2a2e39]/30 transition-colors">
        <td class="py-2 px-4 text-[#d1d4dc] font-sans font-medium">Sharpe Ratio / Sortino Ratio</td>
        <td class="py-2 px-4 text-right text-[#d1d4dc]">
          <span class="text-[#2962ff] font-bold">{fmtNum(summary.sharpe_ratio)}</span> / <span class="text-[#089981] font-bold">{fmtNum(summary.sortino_ratio)}</span>
        </td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
        <td class="py-2 px-4 text-right text-[#787b86]">-</td>
      </tr>
    </tbody>
  </table>
</div>
