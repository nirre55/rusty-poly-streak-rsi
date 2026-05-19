use std::collections::VecDeque;
use chrono::{Datelike, Timelike};
use tracing::debug;

use crate::binance::Candle;
use crate::strategy::{Prediction, Signal, Strategy};

const MAX_WINDOW: usize = 145;
const STRATEGY_NAME: &str = "eth_15m_rules_24_min_votes_1";

// ── RSI Wilder ───────────────────────────────────────────────────────────────

struct RsiState {
    period: usize,
    seed: Vec<(f64, f64)>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    rsi: Option<f64>,
    last_close: Option<f64>,
}

impl RsiState {
    fn new(period: usize) -> Self {
        Self { period, seed: Vec::with_capacity(period), avg_gain: None, avg_loss: None, rsi: None, last_close: None }
    }
    fn update(&mut self, close: f64) {
        if let Some(prev) = self.last_close {
            let d = close - prev;
            let g = d.max(0.0);
            let l = (-d).max(0.0);
            if self.avg_gain.is_none() {
                self.seed.push((g, l));
                if self.seed.len() == self.period {
                    let ag = self.seed.iter().map(|x| x.0).sum::<f64>() / self.period as f64;
                    let al = self.seed.iter().map(|x| x.1).sum::<f64>() / self.period as f64;
                    self.avg_gain = Some(ag);
                    self.avg_loss = Some(al);
                    self.rsi = Some(rsi_val(ag, al));
                }
            } else {
                let p = self.period as f64;
                let ag = (self.avg_gain.unwrap() * (p - 1.0) + g) / p;
                let al = (self.avg_loss.unwrap() * (p - 1.0) + l) / p;
                self.avg_gain = Some(ag);
                self.avg_loss = Some(al);
                self.rsi = Some(rsi_val(ag, al));
            }
        }
        self.last_close = Some(close);
    }
    fn get(&self) -> Option<f64> { self.rsi }
}

fn rsi_val(ag: f64, al: f64) -> f64 {
    if al == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + ag / al) }
}

// ── ATR Wilder ────────────────────────────────────────────────────────────────

struct AtrState {
    period: usize,
    seed: Vec<f64>,
    atr: Option<f64>,
    last_close: Option<f64>,
}

impl AtrState {
    fn new(period: usize) -> Self {
        Self { period, seed: Vec::with_capacity(period), atr: None, last_close: None }
    }
    fn update(&mut self, c: &Candle) {
        if let Some(prev) = self.last_close {
            let tr = (c.high - c.low)
                .max((c.high - prev).abs())
                .max((c.low - prev).abs());
            if self.atr.is_none() {
                self.seed.push(tr);
                if self.seed.len() == self.period {
                    self.atr = Some(self.seed.iter().sum::<f64>() / self.period as f64);
                }
            } else {
                let p = self.period as f64;
                self.atr = Some((self.atr.unwrap() * (p - 1.0) + tr) / p);
            }
        }
        self.last_close = Some(c.close);
    }
    fn pct(&self, close: f64) -> Option<f64> { self.atr.map(|a| a / close) }
    fn raw(&self) -> Option<f64> { self.atr }
}

// ── MACD (histogramme — macd_hist_pct = (macd-signal9)/close) ────────────────

struct MacdState {
    ema12: Option<f64>,
    ema26: Option<f64>,
    signal: Option<f64>,
    hist: Option<f64>,
    n: usize,
}

impl MacdState {
    fn new() -> Self { Self { ema12: None, ema26: None, signal: None, hist: None, n: 0 } }
    fn update(&mut self, close: f64) {
        self.n += 1;
        let a12 = 2.0 / 13.0;
        let a26 = 2.0 / 27.0;
        let a9  = 2.0 / 10.0;
        self.ema12 = Some(match self.ema12 { None => close, Some(e) => e + a12 * (close - e) });
        self.ema26 = Some(match self.ema26 { None => close, Some(e) => e + a26 * (close - e) });
        if self.n >= 26 {
            let m = self.ema12.unwrap() - self.ema26.unwrap();
            self.signal = Some(match self.signal { None => m, Some(s) => s + a9 * (m - s) });
            self.hist = Some(m - self.signal.unwrap());
        }
    }
    fn hist_pct(&self, close: f64) -> Option<f64> { self.hist.map(|h| h / close) }
}

// ── Heikin-Ashi ───────────────────────────────────────────────────────────────

struct HaState {
    ha_open: Option<f64>,
    ha_close: Option<f64>,
}

impl HaState {
    fn new() -> Self { Self { ha_open: None, ha_close: None } }
    fn update(&mut self, c: &Candle) {
        let new_hc = (c.open + c.high + c.low + c.close) / 4.0;
        let new_ho = match (self.ha_open, self.ha_close) {
            (Some(ho), Some(hc)) => (ho + hc) / 2.0,
            _ => (c.open + c.close) / 2.0,
        };
        self.ha_open  = Some(new_ho);
        self.ha_close = Some(new_hc);
    }
    fn ratio(&self, c: &Candle) -> Option<f64> {
        let (ho, hc) = (self.ha_open?, self.ha_close?);
        let hh = c.high.max(ho).max(hc);
        let hl = c.low.min(ho).min(hc);
        let range = hh - hl;
        if range < 1e-12 { return Some(0.0); }
        Some((hc - ho).abs() / range)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmean(v: &[f64]) -> f64 { v.iter().sum::<f64>() / v.len() as f64 }

fn fstd_s(v: &[f64]) -> f64 {
    if v.len() < 2 { return 0.0; }
    let m = fmean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

fn close_z(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n { return None; }
    let v: Vec<f64> = buf.iter().rev().take(n).map(|c| c.close).collect();
    let s = fstd_s(&v);
    Some(if s == 0.0 { 0.0 } else { (v[0] - fmean(&v)) / s })
}

fn vol_z(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n { return None; }
    let v: Vec<f64> = buf.iter().rev().take(n).map(|c| c.volume).collect();
    let s = fstd_s(&v);
    Some(if s == 0.0 { 0.0 } else { (v[0] - fmean(&v)) / s })
}

fn donch_low(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n { return None; }
    let min_low = buf.iter().rev().take(n).map(|c| c.low).fold(f64::INFINITY, f64::min);
    if min_low <= 0.0 { return None; }
    Some(close / min_low - 1.0)
}

fn donch_high(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n { return None; }
    let max_high = buf.iter().rev().take(n).map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
    if max_high <= 0.0 { return None; }
    Some(close / max_high - 1.0)
}

fn bb_pctb(buf: &VecDeque<Candle>) -> Option<f64> {
    if buf.len() < 20 { return None; }
    let v: Vec<f64> = buf.iter().rev().take(20).map(|c| c.close).collect();
    let m = fmean(&v);
    let s = fstd_s(&v);
    if s == 0.0 { return Some(0.5); }
    let upper = m + 2.0 * s;
    let lower = m - 2.0 * s;
    let band = upper - lower;
    if band == 0.0 { return Some(0.5); }
    Some((v[0] - lower) / band)
}

fn body_sum(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n { return None; }
    let s = buf.iter().rev().take(n)
        .map(|c| if c.close != 0.0 { (c.close - c.open) / c.close } else { 0.0 })
        .sum::<f64>();
    Some(s)
}

fn stoch_k(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n { return None; }
    let min_l = buf.iter().rev().take(n).map(|c| c.low).fold(f64::INFINITY, f64::min);
    let max_h = buf.iter().rev().take(n).map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
    let range = max_h - min_l;
    Some(if range == 0.0 { 50.0 } else { (close - min_l) / range * 100.0 })
}

fn ret_n(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    let needed = n + 1;
    if buf.len() < needed { return None; }
    let cur = buf[buf.len() - 1].close;
    let past = buf[buf.len() - 1 - n].close;
    if past == 0.0 { None } else { Some(cur / past - 1.0) }
}

fn dist_sma(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n { return None; }
    let sma = fmean(&buf.iter().rev().take(n).map(|c| c.close).collect::<Vec<_>>());
    if sma == 0.0 { None } else { Some(close / sma - 1.0) }
}

fn cci(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n { return None; }
    let tps: Vec<f64> = buf.iter().rev().take(n).map(|c| (c.high + c.low + c.close) / 3.0).collect();
    let m = fmean(&tps);
    let md = tps.iter().map(|x| (x - m).abs()).sum::<f64>() / n as f64;
    if md == 0.0 { return Some(0.0); }
    Some((tps[0] - m) / (0.015 * md))
}

fn volume_ratio(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n { return None; }
    let sma = buf.iter().rev().take(n).map(|c| c.volume).sum::<f64>() / n as f64;
    if sma < 1e-12 { return None; }
    Some(buf.back()?.volume / sma)
}

fn red_streak(buf: &VecDeque<Candle>) -> f64 {
    let mut count = 0u32;
    for c in buf.iter().rev() {
        if c.close < c.open { count += 1; } else { break; }
    }
    count as f64
}

// ── Features ──────────────────────────────────────────────────────────────────
// 0=donch_low72,    1=upper_wick,     2=range_atr14,   3=rsi7,
// 4=ret24,          5=volume_z96,     6=lower_wick,    7=close_z48,
// 8=body_ratio,     9=donch_high12,   10=stoch_k72,    11=atr14_pct,
// 12=donch_low144,  13=stoch_k12,     14=ha_body_ratio,15=rsi8,
// 16=body_sum12,    17=atr72_pct,     18=bb_pctb,      19=rsi14,
// 20=cci72,         21=rsi21,         22=macd_hist_pct,23=weekday,
// 24=close_z24,     25=close_position,26=dist_sma24,   27=cci12,
// 28=hour,          29=stoch_k24,     30=red_streak,   31=body,
// 32=lower_wick_body,33=volume_ratio20

struct Feats {
    f: [Option<f64>; 34],
}

impl Feats {
    fn get(&self, id: u8) -> Option<f64> { self.f[id as usize] }
}

fn compute_feats(
    buf: &VecDeque<Candle>,
    rsi7: &RsiState, rsi8: &RsiState, rsi14: &RsiState, rsi21: &RsiState,
    atr14: &AtrState, atr72: &AtrState,
    macd: &MacdState,
    ha: &HaState,
) -> Feats {
    let cur = match buf.back() {
        Some(c) => c,
        None => return Feats { f: [None; 34] },
    };
    let close = cur.close;
    let hour    = cur.close_time.hour() as f64;
    let weekday = cur.close_time.weekday().num_days_from_monday() as f64;
    let lower_wick = (cur.open.min(cur.close) - cur.low) / close;
    let upper_wick = (cur.high - cur.open.max(cur.close)) / close;
    let range = cur.high - cur.low;
    let body_ratio     = if range < 1e-12 { 0.0 } else { (cur.close - cur.open).abs() / range };
    let close_position = if range < 1e-12 { 0.5 } else { (cur.close - cur.low) / range };
    let body = if close < 1e-12 { 0.0 } else { (cur.close - cur.open) / close };
    let body_size = (cur.close - cur.open).abs();
    let lower_wick_body = if body_size < 1e-10 { None }
        else { Some((cur.open.min(cur.close) - cur.low) / body_size) };

    let mut f: [Option<f64>; 34] = [None; 34];
    f[0]  = donch_low(buf, 72, close);
    f[1]  = Some(upper_wick);
    f[2]  = atr14.raw().map(|a| if a < 1e-12 { 0.0 } else { range / a });
    f[3]  = rsi7.get();
    f[4]  = ret_n(buf, 24);
    f[5]  = vol_z(buf, 96);
    f[6]  = Some(lower_wick);
    f[7]  = close_z(buf, 48);
    f[8]  = Some(body_ratio);
    f[9]  = donch_high(buf, 12, close);
    f[10] = stoch_k(buf, 72, close);
    f[11] = atr14.pct(close);
    f[12] = donch_low(buf, 144, close);
    f[13] = stoch_k(buf, 12, close);
    f[14] = ha.ratio(cur);
    f[15] = rsi8.get();
    f[16] = body_sum(buf, 12);
    f[17] = atr72.pct(close);
    f[18] = bb_pctb(buf);
    f[19] = rsi14.get();
    f[20] = cci(buf, 72);
    f[21] = rsi21.get();
    f[22] = macd.hist_pct(close);
    f[23] = Some(weekday);
    f[24] = close_z(buf, 24);
    f[25] = Some(close_position);
    f[26] = dist_sma(buf, 24, close);
    f[27] = cci(buf, 12);
    f[28] = Some(hour);
    f[29] = stoch_k(buf, 24, close);
    f[30] = Some(red_streak(buf));
    f[31] = Some(body);
    f[32] = lower_wick_body;
    f[33] = volume_ratio(buf, 20);
    Feats { f }
}

// ── Règles (24) ───────────────────────────────────────────────────────────────
// (green: bool, conditions: [(feat_id, cmp: 0=GE 1=LE 2=EQ, threshold)])

type Rule = (bool, &'static [(u8, u8, f64)]);

fn cmp_ok(val: f64, op: u8, thr: f64) -> bool {
    match op {
        0 => val >= thr,
        1 => val <= thr,
        _ => (val - thr).abs() < 1e-9,
    }
}

fn rule_fires(feats: &Feats, rule: &Rule) -> Option<bool> {
    for &(id, op, thr) in rule.1 {
        let v = feats.get(id)?;
        if !cmp_ok(v, op, thr) { return None; }
    }
    Some(rule.0)
}

static RULES: &[Rule] = &[
    // 1 GREEN micro_next_h1_green_4358
    (true, &[(0,1,0.002929379759),(1,0,0.005230358453),(2,0,1.386145597)]),
    // 2 GREEN micro_next_h1_green_22986
    (true, &[(3,1,18.00845307),(4,0,-0.01422341075_f64),(5,1,0.678389517)]),
    // 3 GREEN micro_next_h1_green_24637
    (true, &[(3,1,20.53343598),(6,1,0.0001510480269),(7,0,-2.194643073_f64)]),
    // 4 GREEN micro_next_h1_green_15764
    (true, &[(3,1,18.00845307),(9,0,-0.01631766978_f64),(8,0,0.761334494)]),
    // 5 GREEN micro_next_h1_green_27887
    (true, &[(10,1,6.338975885),(11,1,0.002139583407),(12,1,0.008900804981)]),
    // 6 GREEN micro_next_h1_green_5771
    (true, &[(13,1,2.190740935),(14,0,0.7392377051),(12,0,0.01135054861)]),
    // 7 GREEN micro_next_h1_green_27692
    (true, &[(15,1,22.26951785),(16,0,-0.009912424083_f64),(8,0,0.761334494)]),
    // 8 GREEN micro_next_h1_green_26864
    (true, &[(15,1,22.26951785),(17,1,0.002457878466),(13,1,12.12790869)]),
    // 9 GREEN micro_next_h1_green_2237
    (true, &[(18,1,-0.1302923821_f64),(11,1,0.002139583407),(19,1,34.24698515)]),
    // 10 GREEN micro_next_h1_green_27610
    (true, &[(15,1,22.26951785),(6,1,0.0001510480269),(20,0,-111.0492288_f64)]),
    // 11 GREEN micro_next_h1_green_22688
    (true, &[(3,1,20.53343598),(11,1,0.002139583407),(29,1,16.25327588)]),
    // 12 GREEN micro_next_h1_green_25275
    (true, &[(0,1,0.0015569182),(21,1,30.19044475),(22,0,-0.00105807534_f64)]),
    // 13 GREEN micro_next_h1_green_2754
    (true, &[(18,1,-0.1302923821_f64),(11,1,0.002139583407),(23,2,6.0)]),
    // 14 GREEN micro_next_h1_green_15491
    (true, &[(3,1,18.00845307),(9,0,-0.01631766978_f64),(0,1,0.0015569182)]),
    // 15 GREEN micro_next_h1_green_26337
    (true, &[(3,1,18.00845307),(13,1,2.190740935),(9,0,-0.02357829884_f64)]),
    // 16 GREEN micro_next_h1_green_16266
    (true, &[(10,1,8.202131158),(28,2,12.0),(21,1,32.08032065)]),
    // 17 GREEN micro_next_h1_green_10425
    (true, &[(15,1,22.26951785),(11,1,0.002521277008),(27,1,-191.4788279_f64)]),
    // 18 GREEN micro_next_h1_green_22587
    (true, &[(24,1,-2.581548268_f64),(11,1,0.002521277008),(12,1,0.006105932389)]),
    // 19 GREEN micro_next_h1_green_26280
    (true, &[(3,1,18.00845307),(26,0,-0.01345743525_f64),(25,1,0.1384615385)]),
    // 20 GREEN struct_streak_rsi_rebound_green_s5_rsi40_atr0.8_body0.75__asia
    //    hour in [0..7] encodé comme hour <= 7
    (true, &[(30,0,5.0),(3,1,40.0),(2,0,0.8),(8,0,0.75),(28,1,7.0)]),
    // 21 GREEN struct_streak_rsi_rebound_green_s2_rsi30_atr0.8_body0.75__hour_6
    (true, &[(30,0,2.0),(3,1,30.0),(2,0,0.8),(8,0,0.75),(28,2,6.0)]),
    // 22 GREEN struct_streak_rsi_rebound_green_s4_rsi30_atr1.5_body0.6__weekday_5
    (true, &[(30,0,4.0),(3,1,30.0),(2,0,1.5),(8,0,0.6),(23,2,5.0)]),
    // 23 GREEN struct_streak_rsi_rebound_green_s3_rsi30_atr0.8_body0.75__weekday_5
    (true, &[(30,0,3.0),(3,1,30.0),(2,0,0.8),(8,0,0.75),(23,2,5.0)]),
    // 24 GREEN struct_wick_volume_rebound_green_rsi30_wick2.0_vol1.0__weekday_5
    (true, &[(31,1,0.0),(3,1,30.0),(32,0,2.0),(33,0,1.0),(23,2,5.0)]),
];

// ── Stratégie ─────────────────────────────────────────────────────────────────

pub struct EthRules24 {
    buffer: VecDeque<Candle>,
    min_votes: u32,
    rsi7: RsiState,
    rsi8: RsiState,
    rsi14: RsiState,
    rsi21: RsiState,
    atr14: AtrState,
    atr72: AtrState,
    macd: MacdState,
    ha: HaState,
    last_votes: (u32, u32),
}

impl EthRules24 {
    pub fn new(min_votes: u32) -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_WINDOW + 1),
            min_votes,
            rsi7: RsiState::new(7),
            rsi8: RsiState::new(8),
            rsi14: RsiState::new(14),
            rsi21: RsiState::new(21),
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
        self.rsi14.update(candle.close);
        self.rsi21.update(candle.close);
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
            &self.rsi7, &self.rsi8, &self.rsi14, &self.rsi21,
            &self.atr14, &self.atr72,
            &self.macd,
            &self.ha,
        );
        let (mut gv, mut rv) = (0u32, 0u32);
        for rule in RULES {
            if let Some(green) = rule_fires(&feats, rule) {
                if green { gv += 1; } else { rv += 1; }
            }
        }
        self.last_votes = (gv, rv);
        (gv, rv)
    }
}

impl Strategy for EthRules24 {
    fn name(&self) -> &str { STRATEGY_NAME }

    fn warmup(&mut self, candle: &Candle) {
        self.feed(candle);
    }

    fn on_closed_candle(&mut self, candle: &Candle) -> Option<Signal> {
        self.feed(candle);
        let (gv, rv) = self.vote();
        let total = gv + rv;

        debug!(
            "[ENSEMBLE] green_votes={} red_votes={} total={} min_votes={}",
            gv, rv, total, self.min_votes
        );

        if total < self.min_votes { return None; }
        if gv == rv { return None; }

        let prediction = if gv > rv { Prediction::Up } else { Prediction::Down };
        let vote_pct = if gv > rv { gv as f64 / total as f64 * 100.0 }
                       else { rv as f64 / total as f64 * 100.0 };

        Some(Signal {
            prediction,
            signal_candle_close_time: candle.close_time,
            rsi: vote_pct,
            strategy_name: self.name().to_string(),
        })
    }

    fn current_rsi(&self) -> Option<f64> { self.rsi7.get() }
    fn current_series(&self) -> Option<bool> { None }
    fn current_atr(&self) -> Option<f64> { self.atr14.raw() }

    fn candle_log_extras(&self) -> String {
        let (gv, rv) = self.last_votes;
        let total = gv + rv;
        if total == 0 {
            return format!("green=0 | red=0 | total=0 | min_votes={}", self.min_votes);
        }
        let dominant = if gv > rv { gv } else { rv };
        let pct = dominant as f64 / total as f64 * 100.0;
        format!(
            "green={} | red={} | total={} | pct={:.1}% | min_votes={}",
            gv, rv, total, pct, self.min_votes
        )
    }
}
