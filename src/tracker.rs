use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval as tick_interval, Duration};
use tracing::{info, warn};

use crate::logger::TradeLogger;
use crate::money::MoneyManager;
use crate::polymarket::PolymarketClient;
use crate::strategy::Prediction;

type PolymarketFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;
const MAX_ORDER_STATUS_FAILURES: u32 = 5;
const STATUS_UNKNOWN: &str = "STATUS_UNKNOWN";

pub trait PolymarketReadClient: Send + Sync {
    fn get_order_status<'a>(&'a self, order_id: &'a str) -> PolymarketFuture<'a, String>;
    fn get_usdc_balance<'a>(&'a self) -> PolymarketFuture<'a, f64>;
}

impl PolymarketReadClient for PolymarketClient {
    fn get_order_status<'a>(&'a self, order_id: &'a str) -> PolymarketFuture<'a, String> {
        Box::pin(async move { PolymarketClient::get_order_status(self, order_id).await })
    }

    fn get_usdc_balance<'a>(&'a self) -> PolymarketFuture<'a, f64> {
        Box::pin(async move { PolymarketClient::get_usdc_balance(self).await })
    }
}

pub fn build_signal_key(strategy_name: &str, slug: &str, prediction: &Prediction) -> String {
    format!(
        "{}:{}:{}",
        strategy_name.trim().to_ascii_lowercase(),
        slug.trim().to_ascii_lowercase(),
        prediction.to_string().to_ascii_uppercase()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PendingTrade {
    trade_id: String,
    order_id: String,
    signal_key: String,
    #[serde(default)]
    prediction: Option<Prediction>,
    #[serde(default)]
    target_close_time_ms: Option<i64>,
    #[serde(default)]
    order_status: Option<String>,
    #[serde(default)]
    status_failures: u32,
    #[serde(default)]
    validation_done: bool,
}

/// Suit les ordres ouverts et met à jour leur `outcome` dans le CSV dès qu'ils
/// atteignent un état terminal (MATCHED / FILLED / CANCELLED / EXPIRED).
///
/// Les ordres dry-run (id préfixé par "dry-run-") sont ignorés silencieusement.
pub struct PositionTracker {
    pending: Mutex<Vec<PendingTrade>>,
    client: Arc<dyn PolymarketReadClient>,
    logger: Arc<TradeLogger>,
    money: Arc<tokio::sync::Mutex<MoneyManager>>,
    state_path: PathBuf,
    trade_amount_pct: f64,
}

impl PositionTracker {
    pub fn new(
        client: Arc<dyn PolymarketReadClient>,
        logger: Arc<TradeLogger>,
        money: Arc<tokio::sync::Mutex<MoneyManager>>,
        logs_dir: &str,
        trade_amount_pct: f64,
    ) -> Self {
        let state_path = PathBuf::from(logs_dir).join("pending_orders.json");
        let pending = Self::load_pending(&state_path);
        if !pending.is_empty() {
            info!(
                "[TRACKER] {} ordres rechargés depuis {}",
                pending.len(),
                state_path.display()
            );
        }
        Self {
            pending: Mutex::new(pending),
            client,
            logger,
            money,
            state_path,
            trade_amount_pct,
        }
    }

    /// Enregistre un ordre pour suivi. Les ordres dry-run sont ignorés.
    pub async fn track(
        &self,
        trade_id: String,
        order_id: String,
        signal_key: String,
        prediction: Prediction,
        target_close_time: DateTime<Utc>,
        order_status: String,
    ) {
        if order_id.starts_with("dry-run-") {
            return;
        }
        let mut pending = self.pending.lock().await;
        if pending
            .iter()
            .any(|t| t.signal_key == signal_key || t.order_id == order_id)
        {
            warn!(
                "[TRACKER] Suivi déjà actif | trade_id={} order_id={} signal_key={}",
                trade_id, order_id, signal_key
            );
            return;
        }
        info!(
            "[TRACKER] Suivi activé | trade_id={} order_id={} signal_key={}",
            trade_id, order_id, signal_key
        );
        pending.push(PendingTrade {
            trade_id,
            order_id,
            signal_key,
            prediction: Some(prediction),
            target_close_time_ms: Some(target_close_time.timestamp_millis()),
            order_status: Some(order_status),
            status_failures: 0,
            validation_done: false,
        });
        if let Err(e) = self.save_pending(&pending) {
            warn!("[TRACKER] Sauvegarde état tracker échouée: {}", e);
        }
    }

    pub async fn is_signal_active(&self, signal_key: &str) -> bool {
        self.pending
            .lock()
            .await
            .iter()
            .any(|trade| trade.signal_key == signal_key)
    }

    pub async fn pending_count(&self) -> usize {
        self.pending.lock().await.len()
    }

    pub async fn validate_with_closed_candle(
        &self,
        candle_close_time: DateTime<Utc>,
        candle_is_green: bool,
    ) {
        let mut pending = self.pending.lock().await;
        let mut changed = false;
        let len_before = pending.len();

        for trade in pending.iter_mut() {
            if trade.validation_done {
                continue;
            }
            let Some(target_close_time_ms) = trade.target_close_time_ms else {
                continue;
            };
            if target_close_time_ms > candle_close_time.timestamp_millis() {
                continue;
            }

            let is_exact_target = target_close_time_ms == candle_close_time.timestamp_millis();
            let outcome = match (&trade.prediction, trade.order_status.as_deref()) {
                (Some(prediction), Some(status))
                    if Self::is_filled_status(status) && is_exact_target =>
                {
                    Some(Self::binance_outcome(prediction, candle_is_green))
                }
                (Some(_), Some(status)) if Self::is_filled_status(status) => {
                    warn!(
                        "[TRACKER] Validation exacte manquée | trade_id={} target_close_time_ms={} current_close_time_ms={}",
                        trade.trade_id,
                        target_close_time_ms,
                        candle_close_time.timestamp_millis()
                    );
                    Some("MISSED_VALIDATION".to_string())
                }
                (_, Some(status)) if Self::is_non_fill_terminal_status(status) => {
                    Some("NO_ENTRY".to_string())
                }
                _ => None,
            };

            if let Some(outcome) = outcome {
                if let Err(e) = self.logger.update_outcome(&trade.trade_id, &outcome) {
                    warn!("[TRACKER] mise a jour outcome echouee: {}", e);
                } else {
                    info!(
                        "[TRACKER] Validation Binance estimee | trade_id={} outcome={}",
                        trade.trade_id, outcome
                    );
                    if matches!(outcome.as_str(), "WIN" | "LOSS" | "NO_ENTRY") {
                        self.money.lock().await.on_outcome(&outcome);
                    }
                    if self.trade_amount_pct > 0.0 && matches!(outcome.as_str(), "WIN" | "LOSS") {
                        let client = self.client.clone();
                        let money = self.money.clone();
                        let pct = self.trade_amount_pct;
                        tokio::spawn(async move {
                            for delay_ms in [300u64, 700, 1500] {
                                tokio::time::sleep(std::time::Duration::from_millis(delay_ms))
                                    .await;
                                match client.get_usdc_balance().await {
                                    Ok(balance) if balance > 0.0 => {
                                        let amount =
                                            (balance * pct / 100.0 * 100.0).floor() / 100.0;
                                        let amount = amount.max(1.0);
                                        info!("[MONEY] Balance post-trade: {:.2}$ → prochain montant = {:.2}$", balance, amount);
                                        money.lock().await.set_base_amount(amount);
                                        return;
                                    }
                                    Ok(_) => warn!(
                                        "[MONEY] Balance USDC encore 0 après {}ms, retry…",
                                        delay_ms
                                    ),
                                    Err(e) => {
                                        warn!("[MONEY] Balance refresh post-trade échoué: {}", e);
                                        return;
                                    }
                                }
                            }
                        });
                    }
                    trade.validation_done = true;
                    changed = true;
                }
            }
        }

        pending.retain(|trade| !Self::can_drop_trade(trade));
        if changed || pending.len() < len_before {
            if let Err(e) = self.save_pending(&pending) {
                warn!("[TRACKER] Sauvegarde état tracker échouée: {}", e);
            }
        }
    }

    /// Boucle de polling en arrière-plan (toutes les 30 secondes).
    /// À lancer avec `tokio::spawn`.
    pub async fn run_poll_loop(self: Arc<Self>) {
        let mut ticker = tick_interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            let pending_count = self.pending.lock().await.len();
            if pending_count == 0 {
                continue;
            }
            info!("[TRACKER] Polling {} ordres ouverts…", pending_count);
            if let Err(e) = self.poll_once().await {
                warn!("[TRACKER] Erreur de polling: {}", e);
            }
        }
    }

    pub async fn poll_once(&self) -> anyhow::Result<()> {
        // Cloner la liste et relacher le lock AVANT les appels reseau
        let trades: Vec<PendingTrade> = self.pending.lock().await.clone();
        let mut still_pending = Vec::new();

        for mut trade in trades {
            match self.client.get_order_status(&trade.order_id).await {
                Ok(status) => {
                    trade.status_failures = 0;
                    let status_changed = trade
                        .order_status
                        .as_deref()
                        .map(|prev| !prev.eq_ignore_ascii_case(&status))
                        .unwrap_or(true);

                    if status_changed {
                        info!(
                            "[TRACKER] trade_id={} order_id={} status={}",
                            trade.trade_id, trade.order_id, status
                        );
                    }
                    if Self::is_terminal_status(&status) {
                        if status_changed {
                            if let Err(e) =
                                self.logger.update_order_status(&trade.trade_id, &status)
                            {
                                warn!("[TRACKER] update_order_status failed: {}", e);
                            } else {
                                trade.order_status = Some(status.clone());
                            }
                        }

                        if Self::is_non_fill_terminal_status(&status) {
                            if let Err(e) = self.logger.update_outcome(&trade.trade_id, "NO_ENTRY")
                            {
                                warn!("[TRACKER] update_outcome failed: {}", e);
                            } else {
                                trade.validation_done = true;
                            }
                        }
                    }
                    still_pending.push(trade);
                }
                Err(e) => {
                    trade.status_failures = trade.status_failures.saturating_add(1);
                    if trade.status_failures >= MAX_ORDER_STATUS_FAILURES {
                        warn!(
                            "[TRACKER] get_order_status({}) failed {} fois; statut marque {}: {}",
                            trade.order_id, trade.status_failures, STATUS_UNKNOWN, e
                        );
                        if let Err(update_err) = self
                            .logger
                            .update_order_status(&trade.trade_id, STATUS_UNKNOWN)
                        {
                            warn!("[TRACKER] update_order_status failed: {}", update_err);
                        }
                        if let Err(update_err) =
                            self.logger.update_outcome(&trade.trade_id, STATUS_UNKNOWN)
                        {
                            warn!("[TRACKER] update_outcome failed: {}", update_err);
                        } else {
                            trade.order_status = Some(STATUS_UNKNOWN.to_string());
                            trade.validation_done = true;
                        }
                    } else {
                        warn!(
                            "[TRACKER] get_order_status({}) failed ({}/{}): {}",
                            trade.order_id, trade.status_failures, MAX_ORDER_STATUS_FAILURES, e
                        );
                    }
                    still_pending.push(trade);
                }
            }
        }

        still_pending.retain(|trade| !Self::can_drop_trade(trade));
        // Reprendre le lock pour mettre à jour l'état
        let mut pending = self.pending.lock().await;
        *pending = still_pending;
        if let Err(e) = self.save_pending(&pending) {
            warn!("[TRACKER] Sauvegarde état tracker échouée: {}", e);
        }
        Ok(())
    }

    fn load_pending(state_path: &PathBuf) -> Vec<PendingTrade> {
        match fs::read_to_string(state_path) {
            Ok(content) => match serde_json::from_str::<Vec<PendingTrade>>(&content) {
                Ok(pending) => pending,
                Err(e) => {
                    warn!(
                        "[TRACKER] pending_orders.json invalide ({}): {}",
                        state_path.display(),
                        e
                    );
                    Vec::new()
                }
            },
            Err(_) => Vec::new(),
        }
    }

    fn save_pending(&self, pending: &[PendingTrade]) -> Result<()> {
        let body = serde_json::to_string_pretty(pending)?;
        fs::write(&self.state_path, body)?;
        Ok(())
    }

    fn is_filled_status(status: &str) -> bool {
        matches!(status.to_ascii_uppercase().as_str(), "MATCHED" | "FILLED")
    }

    fn is_non_fill_terminal_status(status: &str) -> bool {
        matches!(
            status.to_ascii_uppercase().as_str(),
            "CANCELLED" | "EXPIRED" | "UNMATCHED"
        )
    }

    fn is_terminal_status(status: &str) -> bool {
        Self::is_filled_status(status)
            || Self::is_non_fill_terminal_status(status)
            || status.eq_ignore_ascii_case(STATUS_UNKNOWN)
    }

    fn can_drop_trade(trade: &PendingTrade) -> bool {
        trade.validation_done
            && trade
                .order_status
                .as_deref()
                .map(Self::is_terminal_status)
                .unwrap_or(false)
    }

    fn binance_outcome(prediction: &Prediction, candle_is_green: bool) -> String {
        match (prediction, candle_is_green) {
            (Prediction::Up, true) | (Prediction::Down, false) => "WIN".to_string(),
            _ => "LOSS".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PositionTracker;

    #[test]
    fn terminal_status_matching_is_case_insensitive() {
        for status in ["MATCHED", "Matched", "filled", "Cancelled", "EXPIRED"] {
            assert!(PositionTracker::is_terminal_status(status));
        }
    }

    #[test]
    fn open_status_is_not_terminal() {
        assert!(!PositionTracker::is_terminal_status("OPEN"));
    }
}
