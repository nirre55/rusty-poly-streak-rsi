use anyhow::{anyhow, Result};
use chrono::Utc;
use rusty_poly_streak_rsi::config::Config;
use rusty_poly_streak_rsi::polymarket::{
    calculate_available_shares_up_to_price, calculate_limit_order_quote, parse_best_ask_body,
    validate_sufficient_usdc_balance, PolymarketClient,
};

#[derive(Debug)]
struct TokenQuote<'a> {
    side: &'a str,
    token_id: &'a str,
    best_ask: Option<f64>,
    ws_best_ask: Option<f64>,
    limit_price: f64,
    expected_shares: f64,
    effective_usdc: f64,
    adjusted_to_min_size: bool,
    available_shares_at_limit: Option<f64>,
}

struct QuoteRequest<'a> {
    side: &'a str,
    token_id: &'a str,
    requested_usdc: f64,
    min_size: f64,
    limit_price_offset: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let client = PolymarketClient::new(config.clone());
    let http = reqwest::Client::new();

    let interval_ms = interval_ms(&config.interval)?;
    let now_ms = Utc::now().timestamp_millis();
    let next_open_ms = (now_ms / interval_ms + 1) * interval_ms;
    let slug = PolymarketClient::build_slug(&config.polymarket_slug_prefix, next_open_ms);

    println!("Live limit diagnostic (read-only)");
    println!(
        "symbol={} interval={} slug={}",
        config.symbol, config.interval, slug
    );

    let market = client.resolve_market(&slug).await?;
    let balance = client.get_usdc_balance().await?;
    let requested_usdc = if config.trade_amount_pct > 0.0 {
        ((balance * config.trade_amount_pct / 100.0) * 100.0)
            .floor()
            .max(100.0)
            / 100.0
    } else {
        config.trade_amount_usdc
    };

    println!(
        "balance={:.2} USDC requested={:.2} USDC min_size={:.2} shares offset={:.4}",
        balance, requested_usdc, market.order_min_size, config.limit_price_offset
    );

    let up = quote_token(
        &client,
        &http,
        &config.polymarket_api_url,
        QuoteRequest {
            side: "UP",
            token_id: &market.up_token_id,
            requested_usdc,
            min_size: market.order_min_size,
            limit_price_offset: config.limit_price_offset,
        },
    )
    .await?;
    let down = quote_token(
        &client,
        &http,
        &config.polymarket_api_url,
        QuoteRequest {
            side: "DOWN",
            token_id: &market.down_token_id,
            requested_usdc,
            min_size: market.order_min_size,
            limit_price_offset: config.limit_price_offset,
        },
    )
    .await?;

    print_quote(&up, balance);
    print_quote(&down, balance);

    Ok(())
}

async fn quote_token<'a>(
    client: &PolymarketClient,
    http: &reqwest::Client,
    clob_api_base: &str,
    request: QuoteRequest<'a>,
) -> Result<TokenQuote<'a>> {
    let base = clob_api_base.trim_end_matches('/');
    let body = http
        .get(format!("{}/book?token_id={}", base, request.token_id))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let best_ask = parse_best_ask_body(&body);
    let ws_best_ask = client
        .get_best_ask_ws_snapshot(request.token_id, std::time::Duration::from_millis(1500))
        .await;
    let quote = calculate_limit_order_quote(
        request.requested_usdc,
        request.min_size,
        best_ask,
        request.limit_price_offset,
    );
    let available_shares_at_limit =
        calculate_available_shares_up_to_price(&body, quote.limit_price);

    Ok(TokenQuote {
        side: request.side,
        token_id: request.token_id,
        best_ask,
        ws_best_ask,
        limit_price: quote.limit_price,
        expected_shares: quote.expected_shares,
        effective_usdc: quote.effective_usdc,
        adjusted_to_min_size: quote.adjusted_to_min_size,
        available_shares_at_limit,
    })
}

fn print_quote(quote: &TokenQuote<'_>, balance: f64) {
    let balance_status = match validate_sufficient_usdc_balance(quote.effective_usdc, balance) {
        Ok(()) => "OK",
        Err(_) => "INSUFFICIENT_BALANCE",
    };

    println!(
        "{} token={} rest_best_ask={} ws_best_ask={} limit_price={:.4} expected_shares={:.2} available_shares_at_limit={} effective_usdc={:.2} adjusted_to_min_size={} balance_status={}",
        quote.side,
        quote.token_id,
        quote
            .best_ask
            .map(|price| format!("{:.4}", price))
            .unwrap_or_else(|| "NONE".to_string()),
        quote
            .ws_best_ask
            .map(|price| format!("{:.4}", price))
            .unwrap_or_else(|| "NONE".to_string()),
        quote.limit_price,
        quote.expected_shares,
        quote
            .available_shares_at_limit
            .map(|shares| format!("{:.2}", shares))
            .unwrap_or_else(|| "UNKNOWN".to_string()),
        quote.effective_usdc,
        quote.adjusted_to_min_size,
        balance_status
    );
}

fn interval_ms(interval: &str) -> Result<i64> {
    let value = interval
        .strip_suffix('m')
        .ok_or_else(|| anyhow!("interval non supporte pour ce diagnostic: {}", interval))?
        .parse::<i64>()?;
    Ok(value * 60 * 1000)
}
