use chrono::Utc;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::models::candle::Timeframe;
use crate::models::signal::{Signal, SignalAction, SignalStatus};
use crate::models::symbol::Symbol;
use crate::models::tf_compliance::TfComplianceGuard;
use crate::models::tf_qualification::MonthlyQualificationGuard;
use crate::models::tf_spec::{PairTier, TfPairSpec};

#[test]
fn test_tier1_spec_valued_pips_multiplication() {
    let spec = TfPairSpec::from_symbol(&Symbol::new("NZD", "USD"));
    assert_eq!(spec.tier, PairTier::Tier1);
    assert_eq!(spec.value_multiplier, dec!(2.0));

    let vp = spec.pips_to_valued_pips(dec!(50.0));
    assert_eq!(vp, dec!(100.0)); // 50 pips * 2.0 = 100 VP
}

#[test]
fn test_tier4_gold_spec() {
    let spec = TfPairSpec::from_symbol(&Symbol::new("XAU", "USD"));
    assert_eq!(spec.tier, PairTier::Tier4);
    assert_eq!(spec.value_multiplier, dec!(0.5));
    assert_eq!(spec.max_sl_tp_pips, dec!(500.0));

    let vp = spec.pips_to_valued_pips(dec!(100.0));
    assert_eq!(vp, dec!(50.0)); // 100 pips * 0.5 = 50 VP
}

#[test]
fn test_compliance_guard_accepts_valid_pending_order() {
    let signal = Signal {
        id: Uuid::new_v4(),
        symbol: Symbol::new("EUR", "USD"),
        action: SignalAction::BuyLimit,
        timeframe: Timeframe::H1,
        entry_price: dec!(1.08500),
        stop_loss: dec!(1.08200),     // 30 pips SL
        take_profit_1: dec!(1.09100), // 60 pips TP (R:R 1:2.0)
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(2.0),
        confidence_score: 0.9,
        strategy_name: "Pola-N".to_string(),
        rationale: "Retest Higher Low".to_string(),
        status: SignalStatus::Pending,
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
    };

    let result = TfComplianceGuard::validate_signal(&signal);
    assert!(result.is_ok());
}

#[test]
fn test_compliance_guard_rejects_instant_execution() {
    let signal = Signal {
        id: Uuid::new_v4(),
        symbol: Symbol::new("EUR", "USD"),
        action: SignalAction::Buy, // Rejected! Wajib pending order
        timeframe: Timeframe::H1,
        entry_price: dec!(1.08500),
        stop_loss: dec!(1.08200),
        take_profit_1: dec!(1.09100),
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(2.0),
        confidence_score: 0.9,
        strategy_name: "Pola-N".to_string(),
        rationale: "Instant entry".to_string(),
        status: SignalStatus::Active,
        created_at: Utc::now(),
        expires_at: None,
    };

    let result = TfComplianceGuard::validate_signal(&signal);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Pending Order"));
}

#[test]
fn test_compliance_guard_rejects_rr_exceeding_1_to_3() {
    let signal = Signal {
        id: Uuid::new_v4(),
        symbol: Symbol::new("EUR", "USD"),
        action: SignalAction::BuyLimit,
        timeframe: Timeframe::H1,
        entry_price: dec!(1.08500),
        stop_loss: dec!(1.08300),     // 20 pips SL
        take_profit_1: dec!(1.09300), // 80 pips TP (R:R 1:4.0 -> Rejected > 1:3.0)
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(4.0),
        confidence_score: 0.9,
        strategy_name: "Pola-N".to_string(),
        rationale: "Greedy TP".to_string(),
        status: SignalStatus::Pending,
        created_at: Utc::now(),
        expires_at: None,
    };

    let result = TfComplianceGuard::validate_signal(&signal);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("1:3.00"));
}

#[test]
fn test_monthly_qualification_guard_success_and_failure_cases() {
    // Kasus 1: Lulus Sempurna (>= 20 settled signals & >= 300 VP)
    let status_pass = MonthlyQualificationGuard::evaluate_month(25, dec!(450.0));
    assert!(status_pass.is_fully_qualified);
    assert!(status_pass.is_settled_volume_passed);
    assert!(status_pass.is_valued_pips_passed);
    assert!(status_pass.rejection_reasons.is_empty());

    // Kasus 2: Kurang Trade Settled (< 20 signals, e.g. 12)
    let status_low_volume = MonthlyQualificationGuard::evaluate_month(12, dec!(350.0));
    assert!(!status_low_volume.is_fully_qualified);
    assert!(!status_low_volume.is_settled_volume_passed);
    assert!(status_low_volume.is_valued_pips_passed);
    assert_eq!(status_low_volume.rejection_reasons.len(), 1);
    assert!(
        status_low_volume.rejection_reasons[0].contains("Volume sinyal settled (12 sinyal)")
    );

    // Kasus 3: Kurang Valued Pips (< 300 VP, e.g. 180 VP)
    let status_low_vp = MonthlyQualificationGuard::evaluate_month(22, dec!(180.0));
    assert!(!status_low_vp.is_fully_qualified);
    assert!(status_low_vp.is_settled_volume_passed);
    assert!(!status_low_vp.is_valued_pips_passed);
    assert_eq!(status_low_vp.rejection_reasons.len(), 1);
    assert!(status_low_vp.rejection_reasons[0].contains("Akumulasi Valued Pips (180.0 VP)"));
}

#[test]
fn test_concurrent_signals_idempotency_and_anti_martingale() {
    let symbol = Symbol::new("EUR", "USD"); // Tier 2 (min gap 75.0 pips)
    let base_signal = Signal {
        id: Uuid::new_v4(),
        symbol: symbol.clone(),
        action: SignalAction::BuyLimit,
        timeframe: Timeframe::H1,
        entry_price: dec!(1.08000),
        stop_loss: dec!(1.07700),
        take_profit_1: dec!(1.08500),
        take_profit_2: None,
        take_profit_3: None,
        risk_reward_ratio: dec!(1.67),
        confidence_score: 0.9,
        strategy_name: "Pola-N".to_string(),
        rationale: "Base signal".to_string(),
        status: SignalStatus::Pending,
        created_at: Utc::now(),
        expires_at: None,
    };

    // 1. Kasus Duplikat Identik (Idempotency failure)
    let dup_signal = base_signal.clone();
    let err_dup = TfComplianceGuard::validate_concurrent_signals(
        &dup_signal,
        std::slice::from_ref(&base_signal),
    );
    assert!(err_dup.is_err());
    assert!(err_dup
        .unwrap_err()
        .to_string()
        .contains("Sinyal duplikat terdeteksi"));

    // 2. Kasus Anti-Martingale Violation (< 75 pips searah untuk EURUSD)
    let mut close_signal = base_signal.clone();
    close_signal.id = Uuid::new_v4();
    close_signal.entry_price = dec!(1.08300); // 30 pips difference (< 75 pips)
    let err_gap = TfComplianceGuard::validate_concurrent_signals(
        &close_signal,
        std::slice::from_ref(&base_signal),
    );
    assert!(err_gap.is_err());
    assert!(err_gap
        .unwrap_err()
        .to_string()
        .contains("Pelanggaran Anti-Martingale TF"));

    // 3. Kasus Valid Gap (>= 75 pips searah, misal 80 pips)
    let mut valid_signal = base_signal.clone();
    valid_signal.id = Uuid::new_v4();
    valid_signal.entry_price = dec!(1.08800); // 80 pips difference (> 75 pips)
    let ok_gap = TfComplianceGuard::validate_concurrent_signals(
        &valid_signal,
        std::slice::from_ref(&base_signal),
    );
    assert!(ok_gap.is_ok());

    // 4. Kasus Kuota Maksimal 2 Sinyal per Pair Terlampaui
    let mut third_signal = base_signal.clone();
    third_signal.id = Uuid::new_v4();
    third_signal.entry_price = dec!(1.09700);
    let active_two = vec![base_signal.clone(), valid_signal];
    let err_quota = TfComplianceGuard::validate_concurrent_signals(&third_signal, &active_two);
    assert!(err_quota.is_err());
    assert!(err_quota
        .unwrap_err()
        .to_string()
        .contains("Kuota sinyal aktif untuk pair EUR/USD sudah penuh"));
}
