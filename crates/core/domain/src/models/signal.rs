use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::candle::Timeframe;
use super::symbol::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalAction {
    Buy,
    Sell,
    BuyLimit,
    SellLimit,
    BuyStop,
    SellStop,
    ClosePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalStatus {
    Pending,
    Active,
    TargetHit,
    StopLossHit,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    pub id: Uuid,
    pub symbol: Symbol,
    pub action: SignalAction,
    pub timeframe: Timeframe,
    pub entry_price: Decimal,
    pub stop_loss: Decimal,
    pub take_profit_1: Decimal,
    pub take_profit_2: Option<Decimal>,
    pub take_profit_3: Option<Decimal>,
    pub risk_reward_ratio: Decimal,
    pub confidence_score: f32, // 0.0 to 1.0
    pub strategy_name: String,
    pub rationale: String,
    pub status: SignalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Signal {
    pub fn formatted_summary(&self) -> String {
        let action_str = match self.action {
            SignalAction::Buy => "🟢 BUY",
            SignalAction::Sell => "🔴 SELL",
            SignalAction::BuyLimit => "🟢 BUY LIMIT",
            SignalAction::SellLimit => "🔴 SELL LIMIT",
            SignalAction::BuyStop => "🟢 BUY STOP",
            SignalAction::SellStop => "🔴 SELL STOP",
            SignalAction::ClosePosition => "⚪ CLOSE",
        };

        let mut text = format!(
            "📊 FOREX SIGNAL ALERT\n\
             ━━━━━━━━━━━━━━━━━━\n\
             Pair: {}\n\
             Action: {}\n\
             Entry: {}\n\
             Stop Loss: {}\n\
             TP 1: {}\n",
            self.symbol.to_pair_string(),
            action_str,
            self.entry_price,
            self.stop_loss,
            self.take_profit_1
        );

        if let Some(tp2) = self.take_profit_2 {
            text.push_str(&format!("TP 2: {}\n", tp2));
        }
        if let Some(tp3) = self.take_profit_3 {
            text.push_str(&format!("TP 3: {}\n", tp3));
        }

        text.push_str(&format!(
            "R:R Ratio: 1:{:.2}\n\
             Strategy: {}\n\
             Note: {}\n\
             ━━━━━━━━━━━━━━━━━━",
            self.risk_reward_ratio, self.strategy_name, self.rationale
        ));

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_signal_formatted_summary_contains_key_fields() {
        let signal = Signal {
            id: Uuid::new_v4(),
            symbol: Symbol::new("GBP", "USD"),
            action: SignalAction::Buy,
            timeframe: Timeframe::H1,
            entry_price: dec!(1.27500),
            stop_loss: dec!(1.27200),
            take_profit_1: dec!(1.28100),
            take_profit_2: Some(dec!(1.28500)),
            take_profit_3: None,
            risk_reward_ratio: dec!(2.0),
            confidence_score: 0.95,
            strategy_name: "ICT-Silver-Bullet".to_string(),
            rationale: "London session FVG retracement".to_string(),
            status: SignalStatus::Active,
            created_at: Utc::now(),
            expires_at: None,
        };

        let summary = signal.formatted_summary();
        assert!(summary.contains("GBP/USD"));
        assert!(summary.contains("🟢 BUY"));
        assert!(summary.contains("1.27500"));
        assert!(summary.contains("1.27200"));
        assert!(summary.contains("1.28100"));
        assert!(summary.contains("TP 2: 1.28500"));
        assert!(summary.contains("ICT-Silver-Bullet"));
    }
}
