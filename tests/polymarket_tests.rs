use chrono::Utc;
use rusty_poly_streak_rsi::config::{Config, ExecutionMode};
use rusty_poly_streak_rsi::polymarket::{
    calculate_limit_order_quote, parse_best_ask_body, parse_gamma_market_body, MarketInfo,
    PolymarketClient,
};
use rusty_poly_streak_rsi::strategy::{Prediction, Signal};

fn make_config(mode: ExecutionMode) -> Config {
    Config {
        binance_ws_url: "wss://stream.binance.com:9443/ws".to_string(),
        symbol: "btcusdt".to_string(),
        interval: "5m".to_string(),
        execution_mode: mode,
        trade_amount_usdc: 10.0,
        polymarket_api_key: String::new(),
        polymarket_api_secret: String::new(),
        polymarket_api_url: "https://clob-v2.polymarket.com".to_string(),
        logs_dir: "logs".to_string(),
        evm_private_key: None,
        polymarket_funder: None,
        polymarket_signature_type: None,
        strategy: "three_candle_rsi7_reversal".to_string(),
        rsi_overbought: 65.0,
        rsi_oversold: 35.0,
        polymarket_slug_prefix: "btc-updown-5m".to_string(),
        martingale_multiplier: 1.0,
        martingale_max_amount: 0.0,
        trade_amount_pct: 0.0,
        excluded_days: Vec::new(),
        excluded_hours: Vec::new(),
        ensemble_min_votes: 1,
        limit_price_offset: 0.01,
    }
}

fn make_signal(prediction: Prediction) -> Signal {
    Signal {
        prediction,
        signal_candle_close_time: Utc::now(),
        rsi: 72.0,
        strategy_name: "test".to_string(),
    }
}

fn make_market() -> MarketInfo {
    MarketInfo {
        condition_id: "cond_123".to_string(),
        up_token_id: "up_token".to_string(),
        down_token_id: "down_token".to_string(),
        slug: "btc-updown-5m-20240309".to_string(),
        order_min_size: 5.0,
    }
}

// --- build_slug ---

#[test]
fn test_build_slug_known_timestamp() {
    // 1710000000000 ms → Unix 1710000000 s
    let slug = PolymarketClient::build_slug("btc-updown-5m", 1710000000000);
    assert_eq!(slug, "btc-updown-5m-1710000000");
}

#[test]
fn test_build_slug_format_prefix() {
    let slug = PolymarketClient::build_slug("btc-updown-5m", 1710000000000);
    assert!(slug.starts_with("btc-updown-5m-"));
}

#[test]
fn test_build_slug_suffix_is_unix_seconds() {
    // Le suffixe est le timestamp en secondes (pas YYYYMMDD)
    let slug = PolymarketClient::build_slug("btc-updown-5m", 1710000000000);
    let suffix = slug.strip_prefix("btc-updown-5m-").unwrap();
    assert_eq!(suffix, "1710000000");
    assert!(suffix.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_build_slug_ms_to_seconds_truncation() {
    // 1704067200000 ms → 1704067200 s
    let slug = PolymarketClient::build_slug("btc-updown-5m", 1704067200000);
    assert_eq!(slug, "btc-updown-5m-1704067200");
}

#[test]
fn test_build_slug_different_candles_produce_different_slugs() {
    // Deux bougies 5m consécutives (300 000 ms d'écart)
    let slug1 = PolymarketClient::build_slug("btc-updown-5m", 1710000000000);
    let slug2 = PolymarketClient::build_slug("btc-updown-5m", 1710000300000);
    assert_ne!(slug1, slug2);
}

#[test]
fn test_client_uses_configured_clob_api_url_without_trailing_slash() {
    let mut config = make_config(ExecutionMode::DryRun);
    config.polymarket_api_url = "https://example.test/".to_string();
    let client = PolymarketClient::new(config);
    assert_eq!(client.clob_api_base(), "https://example.test");
}

#[test]
fn test_parse_gamma_market_body_maps_reversed_outcomes() {
    let body = r#"[{
        "conditionId":"cond",
        "outcomes":"[\"Down\",\"Up\"]",
        "clobTokenIds":"[\"down-token\",\"up-token\"]",
        "orderMinSize":7
    }]"#;

    let market = parse_gamma_market_body("btc-updown-5m-1710000000", body).unwrap();

    assert_eq!(market.condition_id, "cond");
    assert_eq!(market.up_token_id, "up-token");
    assert_eq!(market.down_token_id, "down-token");
    assert_eq!(market.order_min_size, 7.0);
}

#[test]
fn test_parse_gamma_market_body_rejects_missing_up() {
    let body = r#"[{
        "conditionId":"cond",
        "outcomes":"[\"Down\",\"Flat\"]",
        "clobTokenIds":"[\"down-token\",\"flat-token\"]"
    }]"#;

    let err = parse_gamma_market_body("slug", body).unwrap_err();
    assert!(err.to_string().contains("Outcome 'Up'"));
}

#[test]
fn test_parse_best_ask_body_returns_first_ask_price() {
    let body = r#"{"asks":[{"price":"0.41"},{"price":"0.42"}]}"#;

    assert_eq!(parse_best_ask_body(body), Some(0.41));
}

#[test]
fn test_parse_best_ask_body_returns_none_for_empty_book() {
    assert_eq!(parse_best_ask_body(r#"{"asks":[]}"#), None);
}

#[test]
fn test_calculate_limit_order_quote_uses_offset_and_caps_price() {
    let quote = calculate_limit_order_quote(10.0, 5.0, Some(0.985), 0.02);

    assert_eq!(quote.limit_price, 0.99);
    assert!(!quote.adjusted_to_min_size);
    assert_eq!(quote.effective_usdc, 10.0);
}

#[test]
fn test_calculate_limit_order_quote_adjusts_to_min_size() {
    let quote = calculate_limit_order_quote(1.0, 5.0, Some(0.4), 0.01);

    assert_eq!(quote.limit_price, 0.41000000000000003);
    assert_eq!(quote.effective_usdc, 2.06);
    assert!(quote.adjusted_to_min_size);
}

// --- place_order ---

#[tokio::test]
async fn test_place_order_dryrun_returns_ok() {
    let client = PolymarketClient::new(make_config(ExecutionMode::DryRun));
    let signal = make_signal(Prediction::Up);
    let market = make_market();

    let result = client.place_order(&signal, &market, 10.0).await;
    assert!(result.is_ok(), "DryRun doit retourner Ok");

    let order = result.unwrap();
    assert_eq!(order.status, "DRY_RUN");
    assert!(order.order_id.starts_with("dry-run-"));
}

#[tokio::test]
async fn test_place_order_dryrun_down_signal() {
    let client = PolymarketClient::new(make_config(ExecutionMode::DryRun));
    let signal = make_signal(Prediction::Down);
    let market = make_market();

    let result = client.place_order(&signal, &market, 10.0).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, "DRY_RUN");
}

/// P3 : en mode Market, place_order doit retourner Err (non implémenté en V2)
#[tokio::test]
async fn test_place_order_market_mode_returns_err() {
    let client = PolymarketClient::new(make_config(ExecutionMode::Market));
    let signal = make_signal(Prediction::Up);
    let market = make_market();

    let result = client.place_order(&signal, &market, 10.0).await;
    assert!(
        result.is_err(),
        "Mode Market non implémenté doit retourner Err"
    );
}

/// P3 : en mode Limit, place_order doit retourner Err (non implémenté en V2)
#[tokio::test]
async fn test_place_order_limit_mode_returns_err() {
    let client = PolymarketClient::new(make_config(ExecutionMode::Limit));
    let signal = make_signal(Prediction::Down);
    let market = make_market();

    let result = client.place_order(&signal, &market, 10.0).await;
    assert!(
        result.is_err(),
        "Mode Limit non implémenté doit retourner Err"
    );
}

/// Vérifie que ack_at >= submitted_at (pas de latence négative dans le dry-run)
#[tokio::test]
async fn test_place_order_dryrun_timestamps_ordered() {
    let client = PolymarketClient::new(make_config(ExecutionMode::DryRun));
    let signal = make_signal(Prediction::Up);
    let market = make_market();

    let before = Utc::now();
    let order = client.place_order(&signal, &market, 10.0).await.unwrap();
    assert!(
        order.ack_at >= before,
        "ack_at doit être >= au timestamp avant l'appel"
    );
    assert!(
        order.ack_at >= order.submitted_at,
        "ack_at doit être >= submitted_at"
    );
}
