use chrono::{Datelike, Timelike};
use std::collections::VecDeque;
use tracing::debug;

use crate::binance::Candle;
use crate::strategies::indicators::{AtrState, MacdState, RsiState};
use crate::strategy::{Prediction, Signal, Strategy};

const MAX_WINDOW: usize = 145;
const STRATEGY_NAME: &str = "btc_5m_rules_23_min_votes_1";

struct HaState {
    ha_open: Option<f64>,
    ha_close: Option<f64>,
}

impl HaState {
    fn new() -> Self {
        Self {
            ha_open: None,
            ha_close: None,
        }
    }

    fn update(&mut self, candle: &Candle) {
        let next_close = (candle.open + candle.high + candle.low + candle.close) / 4.0;
        let next_open = match (self.ha_open, self.ha_close) {
            (Some(open), Some(close)) => (open + close) / 2.0,
            _ => (candle.open + candle.close) / 2.0,
        };
        self.ha_open = Some(next_open);
        self.ha_close = Some(next_close);
    }

    fn body(&self, close: f64) -> Option<f64> {
        if close.abs() < 1e-12 {
            return None;
        }
        Some((self.ha_close? - self.ha_open?) / close)
    }
}

fn fmean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn fstd_s(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = fmean(values);
    (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64).sqrt()
}

fn close_z(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let values: Vec<f64> = buf.iter().rev().take(n).map(|c| c.close).collect();
    let std = fstd_s(&values);
    Some(if std == 0.0 {
        0.0
    } else {
        (values[0] - fmean(&values)) / std
    })
}

fn volume_z(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let values: Vec<f64> = buf.iter().rev().take(n).map(|c| c.volume).collect();
    let std = fstd_s(&values);
    Some(if std == 0.0 {
        0.0
    } else {
        (values[0] - fmean(&values)) / std
    })
}

fn donch_low(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let min_low = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min);
    if min_low <= 0.0 {
        None
    } else {
        Some(close / min_low - 1.0)
    }
}

fn donch_high(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let max_high = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    if max_high <= 0.0 {
        None
    } else {
        Some(close / max_high - 1.0)
    }
}

fn bb_pctb(buf: &VecDeque<Candle>) -> Option<f64> {
    if buf.len() < 20 {
        return None;
    }
    let values: Vec<f64> = buf.iter().rev().take(20).map(|c| c.close).collect();
    let mean = fmean(&values);
    let std = fstd_s(&values);
    if std == 0.0 {
        return Some(0.5);
    }
    let upper = mean + 2.0 * std;
    let lower = mean - 2.0 * std;
    let band = upper - lower;
    Some(if band == 0.0 {
        0.5
    } else {
        (values[0] - lower) / band
    })
}

fn body_sum(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    Some(
        buf.iter()
            .rev()
            .take(n)
            .map(|c| {
                if c.close.abs() < 1e-12 {
                    0.0
                } else {
                    (c.close - c.open) / c.close
                }
            })
            .sum(),
    )
}

fn stoch_k(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let min_low = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min);
    let max_high = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let range = max_high - min_low;
    Some(if range == 0.0 {
        50.0
    } else {
        (close - min_low) / range * 100.0
    })
}

fn ret_n(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    let needed = n + 1;
    if buf.len() < needed {
        return None;
    }
    let current = buf[buf.len() - 1].close;
    let past = buf[buf.len() - 1 - n].close;
    if past.abs() < 1e-12 {
        None
    } else {
        Some(current / past - 1.0)
    }
}

fn dist_sma(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let values: Vec<f64> = buf.iter().rev().take(n).map(|c| c.close).collect();
    let sma = fmean(&values);
    if sma.abs() < 1e-12 {
        None
    } else {
        Some(close / sma - 1.0)
    }
}

fn cci(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let typical_prices: Vec<f64> = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| (c.high + c.low + c.close) / 3.0)
        .collect();
    let mean = fmean(&typical_prices);
    let mean_deviation = typical_prices.iter().map(|x| (x - mean).abs()).sum::<f64>() / n as f64;
    if mean_deviation == 0.0 {
        Some(0.0)
    } else {
        Some((typical_prices[0] - mean) / (0.015 * mean_deviation))
    }
}

fn mfi(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n + 1 {
        return None;
    }
    let start = buf.len() - n - 1;
    let (mut positive, mut negative) = (0.0f64, 0.0f64);
    for i in (start + 1)..buf.len() {
        let previous_tp = (buf[i - 1].high + buf[i - 1].low + buf[i - 1].close) / 3.0;
        let current_tp = (buf[i].high + buf[i].low + buf[i].close) / 3.0;
        let raw_money_flow = current_tp * buf[i].volume;
        if current_tp > previous_tp {
            positive += raw_money_flow;
        } else if current_tp < previous_tp {
            negative += raw_money_flow;
        }
    }
    Some(if negative == 0.0 {
        if positive == 0.0 {
            50.0
        } else {
            100.0
        }
    } else {
        100.0 - 100.0 / (1.0 + positive / negative)
    })
}

fn volume_ratio(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let average = buf.iter().rev().take(n).map(|c| c.volume).sum::<f64>() / n as f64;
    if average < 1e-12 {
        None
    } else {
        Some(buf.back()?.volume / average)
    }
}

fn color_streak(buf: &VecDeque<Candle>, green: bool) -> f64 {
    let mut count = 0u32;
    for candle in buf.iter().rev() {
        let matches = if green {
            candle.close > candle.open
        } else {
            candle.close < candle.open
        };
        if matches {
            count += 1;
        } else {
            break;
        }
    }
    count as f64
}

struct Feats {
    f: [Option<f64>; 33],
}

impl Feats {
    fn get(&self, id: u8) -> Option<f64> {
        self.f[id as usize]
    }
}

fn compute_feats(
    buf: &VecDeque<Candle>,
    rsi7: &RsiState,
    rsi8: &RsiState,
    atr14: &AtrState,
    atr72: &AtrState,
    macd: &MacdState,
    ha: &HaState,
) -> Feats {
    let cur = match buf.back() {
        Some(c) => c,
        None => return Feats { f: [None; 33] },
    };
    let close = cur.close;
    let range = cur.high - cur.low;
    let body = if close.abs() < 1e-12 {
        0.0
    } else {
        (cur.close - cur.open) / close
    };
    let body_size = (cur.close - cur.open).abs();
    let lower_wick = if close.abs() < 1e-12 {
        0.0
    } else {
        (cur.open.min(cur.close) - cur.low) / close
    };
    let body_abs_pct = if close.abs() < 1e-12 {
        0.0
    } else {
        body_size / close
    };
    let body_ratio = if range.abs() < 1e-12 {
        0.0
    } else {
        body_size / range
    };
    let lower_wick_body = if body_size < 1e-10 {
        None
    } else {
        Some((cur.open.min(cur.close) - cur.low) / body_size)
    };

    let mut f: [Option<f64>; 33] = [None; 33];
    f[0] = stoch_k(buf, 12, close);
    f[1] = ret_n(buf, 12);
    f[2] = Some(lower_wick);
    f[3] = close_z(buf, 48);
    f[4] = atr72.pct(close);
    f[5] = body_sum(buf, 12);
    f[6] = donch_low(buf, 144, close);
    f[7] = ha.body(close);
    f[8] = Some(body_abs_pct);
    f[9] = cci(buf, 12);
    f[10] = rsi8.get();
    f[11] = body_sum(buf, 6);
    f[12] = volume_z(buf, 96);
    f[13] = stoch_k(buf, 24, close);
    f[14] = macd.hist_pct(close);
    f[15] = atr14
        .raw()
        .map(|atr| if atr < 1e-12 { 0.0 } else { range / atr });
    f[16] = bb_pctb(buf);
    f[17] = Some(cur.close_time.hour() as f64);
    f[18] = dist_sma(buf, 24, close);
    f[19] = donch_low(buf, 72, close);
    f[20] = mfi(buf, 8);
    f[21] = Some(color_streak(buf, true));
    f[22] = close_z(buf, 24);
    f[23] = donch_high(buf, 12, close);
    f[24] = rsi7.get();
    f[25] = Some(body);
    f[26] = lower_wick_body;
    f[27] = volume_ratio(buf, 20);
    f[28] = Some(body_ratio);
    f[29] = Some(cur.close_time.weekday().num_days_from_monday() as f64);
    f[30] = Some(color_streak(buf, false));
    f[31] = atr14.pct(close);
    f[32] = cci(buf, 24);
    Feats { f }
}

type Rule = (bool, &'static [(u8, u8, f64)]);

fn cmp_ok(value: f64, op: u8, threshold: f64) -> bool {
    match op {
        0 => value >= threshold,
        1 => value <= threshold,
        3 => (value - threshold).abs() < 1e-9,
        _ => false,
    }
}

fn rule_fires(feats: &Feats, rule: &Rule) -> Option<bool> {
    for &(feature_id, op, threshold) in rule.1 {
        let value = feats.get(feature_id)?;
        if !cmp_ok(value, op, threshold) {
            return None;
        }
    }
    Some(rule.0)
}

static RULES: &[Rule] = &[
    (
        false,
        &[
            (0, 0, 98.87542775),
            (1, 0, 0.02486548978),
            (2, 1, 0.001638796436),
        ],
    ),
    (
        true,
        &[
            (3, 1, -2.447691672),
            (4, 1, 0.0006406963614),
            (5, 1, -0.004124824725),
        ],
    ),
    (
        true,
        &[
            (6, 1, 0.0006404171868),
            (7, 1, -0.007640254912),
            (8, 0, 0.007312429007),
        ],
    ),
    (
        true,
        &[
            (9, 1, -239.1833565),
            (4, 1, 0.0006406963614),
            (10, 0, 16.98438998),
        ],
    ),
    (
        true,
        &[
            (6, 1, 0.001091384106),
            (11, 1, -0.01817603669),
            (12, 0, 2.919388313),
        ],
    ),
    (
        true,
        &[
            (13, 1, 2.898892702),
            (14, 1, -0.002344176743),
            (15, 0, 1.403339542),
        ],
    ),
    (
        true,
        &[
            (16, 1, -0.2340438348),
            (17, 3, 13.0),
            (18, 0, -0.007134313431),
        ],
    ),
    (
        true,
        &[
            (22, 1, -2.487374544),
            (4, 1, 0.0004654080978),
            (32, 0, -192.0944316),
        ],
    ),
    (
        true,
        &[
            (19, 1, 0.0008266367569),
            (8, 0, 0.007312429007),
            (20, 1, 14.51065799),
        ],
    ),
    (
        false,
        &[(14, 0, 0.00186594868), (0, 0, 97.66150155), (21, 0, 4.0)],
    ),
    (
        false,
        &[(13, 0, 98.04361321), (5, 0, 0.02432418065), (21, 0, 3.0)],
    ),
    (
        true,
        &[
            (9, 1, -239.1833565),
            (22, 0, -2.058232069),
            (2, 1, 0.001092316795),
        ],
    ),
    (
        true,
        &[
            (13, 1, 2.898892702),
            (23, 1, -0.02994954907),
            (20, 1, 14.51065799),
        ],
    ),
    (
        false,
        &[
            (14, 0, 0.00186594868),
            (0, 0, 95.36034773),
            (10, 0, 86.81303658),
        ],
    ),
    (
        true,
        &[
            (9, 1, -209.9116877),
            (4, 1, 0.0007461972567),
            (31, 0, 0.0006619825044),
        ],
    ),
    (
        true,
        &[
            (25, 1, 0.0),
            (24, 1, 25.0),
            (26, 0, 4.0),
            (27, 0, 2.0),
            (29, 3, 2.0),
        ],
    ),
    (
        false,
        &[
            (21, 0, 5.0),
            (24, 0, 75.0),
            (15, 0, 1.5),
            (28, 0, 0.75),
            (29, 3, 3.0),
        ],
    ),
    (
        false,
        &[
            (21, 0, 4.0),
            (24, 0, 75.0),
            (15, 0, 1.0),
            (28, 0, 0.75),
            (17, 3, 1.0),
        ],
    ),
    (
        false,
        &[
            (21, 0, 4.0),
            (24, 0, 75.0),
            (15, 0, 0.8),
            (28, 0, 0.75),
            (17, 3, 11.0),
        ],
    ),
    (
        true,
        &[
            (30, 0, 3.0),
            (24, 1, 30.0),
            (15, 0, 1.5),
            (28, 0, 0.75),
            (17, 3, 21.0),
        ],
    ),
    (
        false,
        &[
            (21, 0, 6.0),
            (24, 0, 70.0),
            (15, 0, 0.8),
            (28, 0, 0.75),
            (29, 3, 5.0),
        ],
    ),
    (
        true,
        &[
            (30, 0, 5.0),
            (24, 1, 30.0),
            (15, 0, 1.5),
            (28, 0, 0.75),
            (17, 0, 21.0),
            (17, 1, 23.0),
        ],
    ),
    (
        true,
        &[
            (25, 1, 0.0),
            (24, 1, 30.0),
            (26, 0, 1.5),
            (27, 0, 1.5),
            (17, 3, 22.0),
        ],
    ),
];

pub struct BtcRules23 {
    buffer: VecDeque<Candle>,
    min_votes: u32,
    rsi7: RsiState,
    rsi8: RsiState,
    atr14: AtrState,
    atr72: AtrState,
    macd: MacdState,
    ha: HaState,
    last_votes: (u32, u32),
}

impl BtcRules23 {
    pub fn new(min_votes: u32) -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_WINDOW + 1),
            min_votes,
            rsi7: RsiState::new(7),
            rsi8: RsiState::new(8),
            atr14: AtrState::new(14),
            atr72: AtrState::new(72),
            macd: MacdState::new(),
            ha: HaState::new(),
            last_votes: (0, 0),
        }
    }

    fn feed(&mut self, candle: &Candle) {
        self.rsi7.update(candle.close);
        self.rsi8.update(candle.close);
        self.atr14.update(candle);
        self.atr72.update(candle);
        self.macd.update(candle.close);
        self.ha.update(candle);
        self.buffer.push_back(candle.clone());
        if self.buffer.len() > MAX_WINDOW {
            self.buffer.pop_front();
        }
    }

    fn vote(&mut self) -> (u32, u32) {
        let feats = compute_feats(
            &self.buffer,
            &self.rsi7,
            &self.rsi8,
            &self.atr14,
            &self.atr72,
            &self.macd,
            &self.ha,
        );
        let (mut green_votes, mut red_votes) = (0u32, 0u32);
        for rule in RULES {
            if let Some(green) = rule_fires(&feats, rule) {
                if green {
                    green_votes += 1;
                } else {
                    red_votes += 1;
                }
            }
        }
        self.last_votes = (green_votes, red_votes);
        (green_votes, red_votes)
    }
}

impl Strategy for BtcRules23 {
    fn name(&self) -> &str {
        STRATEGY_NAME
    }

    fn warmup(&mut self, candle: &Candle) {
        self.feed(candle);
    }

    fn on_closed_candle(&mut self, candle: &Candle) -> Option<Signal> {
        self.feed(candle);
        if candle.open == candle.close {
            self.last_votes = (0, 0);
            return None;
        }

        let (green_votes, red_votes) = self.vote();
        let total = green_votes + red_votes;
        debug!(
            "[ENSEMBLE] green_votes={} red_votes={} total={} min_votes={}",
            green_votes, red_votes, total, self.min_votes
        );

        if total < self.min_votes || green_votes == red_votes {
            return None;
        }

        let prediction = if green_votes > red_votes {
            Prediction::Up
        } else {
            Prediction::Down
        };
        let dominant_votes = green_votes.max(red_votes);
        Some(Signal {
            prediction,
            signal_candle_close_time: candle.close_time,
            rsi: dominant_votes as f64 / total as f64 * 100.0,
            strategy_name: self.name().to_string(),
        })
    }

    fn current_rsi(&self) -> Option<f64> {
        self.rsi7.get()
    }

    fn current_series(&self) -> Option<bool> {
        None
    }

    fn current_atr(&self) -> Option<f64> {
        self.atr14.raw()
    }

    fn candle_log_extras(&self) -> String {
        let (green_votes, red_votes) = self.last_votes;
        let total = green_votes + red_votes;
        if total == 0 {
            return format!("green=0 | red=0 | total=0 | min_votes={}", self.min_votes);
        }
        let dominant_votes = green_votes.max(red_votes);
        let vote_pct = dominant_votes as f64 / total as f64 * 100.0;
        format!(
            "green={} | red={} | total={} | pct={:.1}% | min_votes={}",
            green_votes, red_votes, total, vote_pct, self.min_votes
        )
    }
}
