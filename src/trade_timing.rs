use chrono::{DateTime, Utc};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeLatencies {
    pub signal_to_submit_start_ms: i64,
    pub submit_start_to_ack_ms: i64,
    pub signal_to_ack_ms: i64,
    pub trade_open_to_order_ack_ms: i64,
}

impl TradeLatencies {
    pub fn from_times(
        signal_received_at: DateTime<Utc>,
        order_submit_started_at: DateTime<Utc>,
        order_ack_at: DateTime<Utc>,
        candle_close_time: DateTime<Utc>,
    ) -> Self {
        Self {
            signal_to_submit_start_ms: clamp_non_negative(
                "Latence signal→submit négative",
                order_submit_started_at - signal_received_at,
            ),
            submit_start_to_ack_ms: clamp_non_negative(
                "Latence submit→ack négative",
                order_ack_at - order_submit_started_at,
            ),
            signal_to_ack_ms: clamp_non_negative(
                "Latence signal→ack négative",
                order_ack_at - signal_received_at,
            ),
            trade_open_to_order_ack_ms: clamp_trade_open_latency(order_ack_at - candle_close_time),
        }
    }
}

fn clamp_non_negative(label: &str, duration: chrono::Duration) -> i64 {
    let ms = duration.num_milliseconds();
    if ms < 0 {
        warn!("{} ({}ms) — désync NTP ?", label, ms);
    }
    ms.max(0)
}

fn clamp_trade_open_latency(duration: chrono::Duration) -> i64 {
    let ms = duration.num_milliseconds();
    if ms < -2_000 {
        warn!(
            "Latence bougie→ack très négative ({}ms) — désync horloge Binance/locale ?",
            ms
        );
    }
    ms.max(0)
}

#[cfg(test)]
mod tests {
    use super::TradeLatencies;
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn computes_positive_latencies() {
        let signal = Utc.with_ymd_and_hms(2026, 5, 19, 10, 0, 0).unwrap();
        let submit = signal + Duration::milliseconds(25);
        let ack = submit + Duration::milliseconds(75);
        let candle_close = signal - Duration::milliseconds(500);

        let latencies = TradeLatencies::from_times(signal, submit, ack, candle_close);

        assert_eq!(latencies.signal_to_submit_start_ms, 25);
        assert_eq!(latencies.submit_start_to_ack_ms, 75);
        assert_eq!(latencies.signal_to_ack_ms, 100);
        assert_eq!(latencies.trade_open_to_order_ack_ms, 600);
    }

    #[test]
    fn clamps_negative_latencies_to_zero() {
        let signal = Utc.with_ymd_and_hms(2026, 5, 19, 10, 0, 0).unwrap();
        let submit = signal - Duration::milliseconds(25);
        let ack = signal - Duration::milliseconds(50);
        let candle_close = signal + Duration::milliseconds(500);

        let latencies = TradeLatencies::from_times(signal, submit, ack, candle_close);

        assert_eq!(latencies.signal_to_submit_start_ms, 0);
        assert_eq!(latencies.submit_start_to_ack_ms, 0);
        assert_eq!(latencies.signal_to_ack_ms, 0);
        assert_eq!(latencies.trade_open_to_order_ack_ms, 0);
    }
}
