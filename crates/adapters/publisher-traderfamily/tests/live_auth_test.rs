use publisher_traderfamily::{TraderFamilyConfig, TraderFamilyPublisher};

#[tokio::test]
async fn test_live_traders_family_auto_login() {
    let config = TraderFamilyConfig {
        base_url: "https://app3.tradersfamily.app".to_string(),
        auth_token: String::new(),
        channel_id: "341232".to_string(),
        user_agent: "Dart/3.4 (dart:io)".to_string(),
        email: "ihza2karunia@gmail.com".to_string(),
        password: "Fikrizaki2!".to_string(),
    };

    let publisher = TraderFamilyPublisher::new(config).expect("Publisher must build");
    let token = publisher
        .get_valid_token()
        .await
        .expect("Auto-login must succeed");

    assert!(!token.is_empty(), "Token must not be empty");
    assert!(token.starts_with("eyJ"), "Token must be a valid JWT");
    println!(
        "🎉 Test Auto-login Live Berhasil! JWT Token prefix: {}...",
        &token[..15]
    );
}
