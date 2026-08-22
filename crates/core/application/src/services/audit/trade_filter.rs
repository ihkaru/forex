use chrono::Datelike;
use domain::ports::audit::{
    FilteredTradesSummary, PaginatedTradesResponse, SortDirection, TradeActionFilter,
    TradeAuditItem, TradeExitFilter, TradeFilterQuery, TradeResultFilter, TradeSortField,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Modul filtering, sorting, dan paginasi transaksi backtest audit secara deterministik.
pub struct TradeSearchFilter;

impl TradeSearchFilter {
    /// Memfilter dan memaginasi daftar transaksi berdasarkan query parameter.
    pub fn filter_and_paginate(
        mut filtered: Vec<TradeAuditItem>,
        query: &TradeFilterQuery,
    ) -> PaginatedTradesResponse {
        // 0. Search Query Text & Smart Operator Filter (e.g. ">100", "<0", "pnl>50", "vp>100", "duration>24")
        if let Some(ref q) = query.search_query {
            let clean_q = q.trim().to_lowercase();
            if !clean_q.is_empty() {
                if let Some(rest) = clean_q.strip_prefix(">=") {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.pnl_pips >= num || t.valued_pips >= num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix("<=") {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.pnl_pips <= num || t.valued_pips <= num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix('>') {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.pnl_pips > num || t.valued_pips > num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix('<') {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.pnl_pips < num || t.valued_pips < num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix("pnl>") {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.pnl_pips > num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix("pnl<") {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.pnl_pips < num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix("vp>") {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.valued_pips > num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix("vp<") {
                    if let Ok(num) = rest.trim().parse::<Decimal>() {
                        filtered.retain(|t| t.valued_pips < num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix("hours>") {
                    if let Ok(num) = rest.trim().parse::<i64>() {
                        filtered.retain(|t| t.duration_hours > num);
                    }
                } else if let Some(rest) = clean_q.strip_prefix("hours<") {
                    if let Ok(num) = rest.trim().parse::<i64>() {
                        filtered.retain(|t| t.duration_hours < num);
                    }
                } else {
                    filtered.retain(|t| {
                        t.id.to_lowercase().contains(&clean_q)
                            || t.action.to_lowercase().contains(&clean_q)
                            || t.exit_reason.to_lowercase().contains(&clean_q)
                            || t.entry_price.to_string().contains(&clean_q)
                            || t.exit_price.to_string().contains(&clean_q)
                            || t.pnl_pips.to_string().contains(&clean_q)
                            || t.valued_pips.to_string().contains(&clean_q)
                            || t.open_time.to_rfc3339().to_lowercase().contains(&clean_q)
                            || t.close_time.to_rfc3339().to_lowercase().contains(&clean_q)
                    });
                }
            }
        }

        // 1. Filter by Action / Direction
        if let Some(ref action_filter) = query.action {
            match action_filter {
                TradeActionFilter::Buy => {
                    filtered.retain(|t| t.action.to_uppercase().contains("BUY"));
                }
                TradeActionFilter::Sell => {
                    filtered.retain(|t| t.action.to_uppercase().contains("SELL"));
                }
                TradeActionFilter::All => {}
            }
        }

        // 2. Filter by Result (Win / Loss)
        if let Some(ref res_filter) = query.result {
            match res_filter {
                TradeResultFilter::Win => {
                    filtered.retain(|t| t.pnl_pips > dec!(0.0));
                }
                TradeResultFilter::Loss => {
                    filtered.retain(|t| t.pnl_pips <= dec!(0.0));
                }
                TradeResultFilter::All => {}
            }
        }

        // 3. Filter by Exit Reason
        if let Some(ref exit_filter) = query.exit_reason {
            match exit_filter {
                TradeExitFilter::TakeProfit => {
                    filtered.retain(|t| t.exit_reason.to_uppercase().contains("TAKE_PROFIT"));
                }
                TradeExitFilter::StopLoss => {
                    filtered.retain(|t| t.exit_reason.to_uppercase().contains("STOP_LOSS"));
                }
                TradeExitFilter::Expired => {
                    filtered.retain(|t| t.exit_reason.to_uppercase().contains("EXPIRED"));
                }
                TradeExitFilter::All => {}
            }
        }

        // 4. Filter by Year & Month
        if let Some(y) = query.year {
            filtered.retain(|t| t.close_time.year() == y);
        }
        if let Some(m) = query.month {
            filtered.retain(|t| t.close_time.month() == m);
        }

        // 5. Filter by PnL Comparison (> / < / >= / <=)
        if let Some(pnl_gt) = query.pnl_gt {
            filtered.retain(|t| t.pnl_pips > pnl_gt);
        }
        if let Some(pnl_gte) = query.pnl_gte {
            filtered.retain(|t| t.pnl_pips >= pnl_gte);
        }
        if let Some(min_pnl) = query.min_pnl_pips {
            filtered.retain(|t| t.pnl_pips >= min_pnl);
        }
        if let Some(pnl_lt) = query.pnl_lt {
            filtered.retain(|t| t.pnl_pips < pnl_lt);
        }
        if let Some(pnl_lte) = query.pnl_lte {
            filtered.retain(|t| t.pnl_pips <= pnl_lte);
        }
        if let Some(max_pnl) = query.max_pnl_pips {
            filtered.retain(|t| t.pnl_pips <= max_pnl);
        }

        // 6. Filter by Valued Pips Comparison (> / < / >= / <=)
        if let Some(vp_gt) = query.vp_gt {
            filtered.retain(|t| t.valued_pips > vp_gt);
        }
        if let Some(vp_gte) = query.vp_gte {
            filtered.retain(|t| t.valued_pips >= vp_gte);
        }
        if let Some(min_vp) = query.min_valued_pips {
            filtered.retain(|t| t.valued_pips >= min_vp);
        }
        if let Some(vp_lt) = query.vp_lt {
            filtered.retain(|t| t.valued_pips < vp_lt);
        }
        if let Some(vp_lte) = query.vp_lte {
            filtered.retain(|t| t.valued_pips <= vp_lte);
        }

        // 7. Filter by Price Comparison (> / <)
        if let Some(price_gt) = query.price_gt {
            filtered.retain(|t| t.entry_price > price_gt || t.exit_price > price_gt);
        }
        if let Some(price_lt) = query.price_lt {
            filtered.retain(|t| t.entry_price < price_lt || t.exit_price < price_lt);
        }

        // 8. Filter by Holding Duration (> / < / min / max)
        if let Some(dur_gt) = query.duration_gt {
            filtered.retain(|t| t.duration_hours > dur_gt);
        }
        if let Some(min_d) = query.min_duration_hours {
            filtered.retain(|t| t.duration_hours >= min_d);
        }
        if let Some(dur_lt) = query.duration_lt {
            filtered.retain(|t| t.duration_hours < dur_lt);
        }
        if let Some(max_d) = query.max_duration_hours {
            filtered.retain(|t| t.duration_hours <= max_d);
        }

        // Calculate Filtered Aggregate Summary
        let matched_count = filtered.len();
        let mut win_count = 0;
        let mut loss_count = 0;
        let mut gross_profit = dec!(0.0);
        let mut gross_loss = dec!(0.0);
        let mut total_pnl = dec!(0.0);
        let mut total_vp = dec!(0.0);

        for t in &filtered {
            total_pnl += t.pnl_pips;
            total_vp += t.valued_pips;
            if t.pnl_pips > dec!(0.0) {
                win_count += 1;
                gross_profit += t.pnl_pips;
            } else {
                loss_count += 1;
                gross_loss += t.pnl_pips.abs();
            }
        }

        let win_rate = if matched_count > 0 {
            Decimal::from(win_count) / Decimal::from(matched_count) * dec!(100.0)
        } else {
            dec!(0.0)
        };

        let pf = if gross_loss > dec!(0.0) {
            gross_profit / gross_loss
        } else if gross_profit > dec!(0.0) {
            dec!(99.0)
        } else {
            dec!(0.0)
        };

        let avg_trade = if matched_count > 0 {
            total_pnl / Decimal::from(matched_count)
        } else {
            dec!(0.0)
        };

        let summary = FilteredTradesSummary {
            matched_trades: matched_count,
            winning_trades: win_count,
            losing_trades: loss_count,
            win_rate_pct: win_rate,
            total_raw_pips: total_pnl,
            total_valued_pips: total_vp,
            gross_profit_pips: gross_profit,
            gross_loss_pips: gross_loss,
            profit_factor: pf,
            avg_trade_pips: avg_trade,
        };

        // 8. Sorting
        let sort_field = query.sort_by.clone().unwrap_or(TradeSortField::CloseTime);
        let is_desc = query.sort_direction != Some(SortDirection::Asc);

        filtered.sort_by(|a, b| {
            let ordering = match sort_field {
                TradeSortField::Index => a.open_epoch.cmp(&b.open_epoch),
                TradeSortField::CloseTime => a.close_time.cmp(&b.close_time),
                TradeSortField::OpenTime => a.open_time.cmp(&b.open_time),
                TradeSortField::Action => a.action.cmp(&b.action),
                TradeSortField::OpenPrice => a.entry_price.cmp(&b.entry_price),
                TradeSortField::ClosePrice => a.exit_price.cmp(&b.exit_price),
                TradeSortField::PnlPips => a.pnl_pips.cmp(&b.pnl_pips),
                TradeSortField::ValuedPips => a.valued_pips.cmp(&b.valued_pips),
                TradeSortField::DurationHours => a.duration_hours.cmp(&b.duration_hours),
                TradeSortField::ExitReason => a.exit_reason.cmp(&b.exit_reason),
            };
            if is_desc {
                ordering.reverse()
            } else {
                ordering
            }
        });

        // 9. Pagination
        let page_size = query.page_size.clamp(1, 500);
        let current_page = query.page.max(1);
        let total_pages = if matched_count > 0 {
            matched_count.div_ceil(page_size)
        } else {
            1
        };

        let start_idx = (current_page - 1) * page_size;
        let paged_trades = if start_idx < matched_count {
            let end_idx = (start_idx + page_size).min(matched_count);
            filtered[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        PaginatedTradesResponse {
            symbol: query.symbol.clone(),
            total_records: matched_count,
            total_pages,
            current_page,
            page_size,
            has_next_page: current_page < total_pages,
            has_prev_page: current_page > 1,
            summary,
            trades: paged_trades,
        }
    }
}
