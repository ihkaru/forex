use chrono::Utc;
use domain::models::{
    AppConfig, Signal, SignalAction, SignalStatus, Symbol, TfComplianceGuard, Timeframe,
};
use domain::ports::SignalPublisherPort;
use publisher_traderfamily::{TraderFamilyConfig, TraderFamilyPublisher};
use rust_decimal_macros::dec;
use uuid::Uuid;

pub async fn run_compliance_and_publish(config: &AppConfig) -> anyhow::Result<()> {
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!("🛡️ 5. UJI COBA VALIDASI KEPATUHAN TF COMPLIANCE GUARD");
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════"
    );

    let sample_signal = Signal {
        id: Uuid::new_v4(),
        symbol: Symbol::new("NZD", "USD"),
        action: SignalAction::BuyLimit,
        timeframe: Timeframe::H1,
        entry_price: dec!(0.60500),
        stop_loss: dec!(0.60250),     // 25 pips SL (Tier 1 min 10 max 200)
        take_profit_1: dec!(0.61000), // 50 pips TP (R:R 1:2.0)
        take_profit_2: Some(dec!(0.61200)),
        take_profit_3: None,
        risk_reward_ratio: dec!(2.0),
        confidence_score: 0.94,
        strategy_name: "TF-Pola-N-Core-v1".to_string(),
        rationale: "Retest Higher Low L2 pada key level support H1".to_string(),
        status: SignalStatus::Pending,
        created_at: Utc::now(),
        expires_at: Some(
            Utc::now() + chrono::Duration::hours(config.strategy_pola_n.default_expiration_hours),
        ),
    };

    match TfComplianceGuard::validate_signal(&sample_signal) {
        Ok(_) => {
            println!("✅ Sinyal lolos verifikasi TfComplianceGuard 100% (Zero-Penalty Guarantee)")
        }
        Err(e) => println!("❌ Sinyal ditolak: {}", e),
    }

    println!("\n[PREVIEW FORMAT SINYAL RESMI CHANNEL TRADERS FAMILY]:\n");
    println!("{}", sample_signal.formatted_summary());

    // Test Publishing via Publisher Adapter (Menggunakan Kredensial Config)
    println!("\n[MENGIRIMKAN SINYAL UJI COBA VIA PUBLISHER PORT]...");
    let tf_publisher = TraderFamilyPublisher::new(TraderFamilyConfig {
        base_url: config.traders_family.api_base_url.clone(),
        auth_token: config.traders_family.auth_token.clone(),
        channel_id: config.traders_family.channel_id.clone(),
        user_agent: config.traders_family.user_agent.clone(),
        email: config.traders_family.email.clone(),
        password: config.traders_family.password.clone(),
    })?;

    let receipt = tf_publisher.publish_signal(&sample_signal).await?;
    println!(
        "✅ Selesai! Post ID: {} berhasil diarahkan ke channel: {}\n",
        receipt.external_post_id, receipt.channel_target
    );

    Ok(())
}
