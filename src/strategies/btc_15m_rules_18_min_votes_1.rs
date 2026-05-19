use std::collections::VecDeque;
use chrono::Datelike;
use tracing::debug;

use crate::binance::Candle;
use crate::strategy::{Prediction, Signal, Strategy};

const MAX_WINDOW: usize = 100;
const STRATEGY_NAME: &str = "btc_15m_rules_18_min_votes_1";

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

// ── MACD (ligne seulement — macd_pct = (ema12-ema26)/close) ──────────────────

struct MacdState {
    ema12: Option<f64>,
    ema26: Option<f64>,
    n: usize,
}

impl MacdState {
    fn new() -> Self { Self { ema12: None, ema26: None, n: 0 } }
    fn update(&mut self, close: f64) {
        self.n += 1;
        let a12 = 2.0 / 13.0;
        let a26 = 2.0 / 27.0;
        self.ema12 = Some(match self.ema12 { None => close, Some(e) => e + a12 * (close - e) });
        self.ema26 = Some(match self.ema26 { None => close, Some(e) => e + a26 * (close - e) });
    }
    fn line_pct(&self, close: f64) -> Option<f64> {
        if self.n < 26 { return None; }
        Some((self.ema12.unwrap() - self.ema26.unwrap()) / close)
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

fn mfi(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n + 1 { return None; }
    let start = buf.len() - n - 1;
    let (mut pos, mut neg) = (0.0f64, 0.0f64);
    for i in (start + 1)..buf.len() {
        let prev_tp = (buf[i-1].high + buf[i-1].low + buf[i-1].close) / 3.0;
        let curr_tp = (buf[i].high + buf[i].low + buf[i].close) / 3.0;
        let rmf = curr_tp * buf[i].volume;
        if curr_tp > prev_tp { pos += rmf; }
        else if curr_tp < prev_tp { neg += rmf; }
    }
    Some(if neg == 0.0 { if pos == 0.0 { 50.0 } else { 100.0 } }
         else { 100.0 - 100.0 / (1.0 + pos / neg) })
}

fn volume_ratio(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n { return None; }
    let sma = buf.iter().rev().take(n).map(|c| c.volume).sum::<f64>() / n as f64;
    if sma < 1e-12 { return None; }
    Some(buf.back()?.volume / sma)
}

// ── Features ──────────────────────────────────────────────────────────────────
// 0=close_z24,      1=lower_wick,     2=body_sum12,    3=stoch_k12,
// 4=weekday,        5=bb_pctb,        6=body_ratio,    7=range_atr14,
// 8=cci12,          9=lower_wick_body,10=dist_sma24,   11=atr14_pct,
// 12=rsi8,          13=upper_wick,    14=stoch_k24,    15=body_abs_pct,
// 16=macd_pct,      17=ret24,         18=close_position,19=rsi7,
// 20=cci24,         21=mfi8,          22=volume_ratio20,23=volume_z96

struct Feats {
    f: [Option<f64>; 24],
}

impl Feats {
    fn get(&self, id: u8) -> Option<f64> { self.f[id as usize] }
}

fn compute_feats(
    buf: &VecDeque<Candle>,
    rsi7: &RsiState, rsi8: &RsiState,
    atr14: &AtrState,
    macd: &MacdState,
) -> Feats {
    let cur = match buf.back() {
        Some(c) => c,
        None => return Feats { f: [None; 24] },
    };
    let close = cur.close;
    let weekday = cur.close_time.weekday().num_days_from_monday() as f64;
    let lower_wick = (cur.open.min(cur.close) - cur.low) / close;
    let upper_wick = (cur.high - cur.open.max(cur.close)) / close;
    let range = cur.high - cur.low;
    let body_ratio = if range < 1e-12 { 0.0 } else { (cur.close - cur.open).abs() / range };
    let close_position = if range < 1e-12 { 0.5 } else { (cur.close - cur.low) / range };
    let body_abs_pct = if close < 1e-12 { 0.0 } else { (cur.close - cur.open).abs() / close };
    let body_size = (cur.close - cur.open).abs();
    let lower_wick_body = if body_size < 1e-10 { None }
        else { Some((cur.open.min(cur.close) - cur.low) / body_size) };

    let mut f: [Option<f64>; 24] = [None; 24];
    f[0]  = close_z(buf, 24);
    f[1]  = Some(lower_wick);
    f[2]  = body_sum(buf, 12);
    f[3]  = stoch_k(buf, 12, close);
    f[4]  = Some(weekday);
    f[5]  = bb_pctb(buf);
    f[6]  = Some(body_ratio);
    f[7]  = atr14.raw().map(|a| if a < 1e-12 { 0.0 } else { range / a });
    f[8]  = cci(buf, 12);
    f[9]  = lower_wick_body;
    f[10] = dist_sma(buf, 24, close);
    f[11] = atr14.pct(close);
    f[12] = rsi8.get();
    f[13] = Some(upper_wick);
    f[14] = stoch_k(buf, 24, close);
    f[15] = Some(body_abs_pct);
    f[16] = macd.line_pct(close);
    f[17] = ret_n(buf, 24);
    f[18] = Some(close_position);
    f[19] = rsi7.get();
    f[20] = cci(buf, 24);
    f[21] = mfi(buf, 8);
    f[22] = volume_ratio(buf, 20);
    f[23] = vol_z(buf, 96);
    Feats { f }
}

// ── Règles (18) ───────────────────────────────────────────────────────────────
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
    // 1 GREEN micro_next_h1_green_10688
    (true,  &[(0,1,-2.346311048_f64),(1,1,2.645309807e-05),(2,0,-0.007253030737)]),
    // 2 GREEN micro_next_h1_green_23009
    (true,  &[(3,1,0.5066458518),(4,2,5.0),(1,0,9.290936623e-08)]),
    // 3 GREEN micro_next_h1_green_34382
    (true,  &[(5,1,-0.01125914436_f64),(6,0,0.9656401664),(7,1,1.586225659)]),
    // 4 GREEN micro_next_h1_green_31644
    (true,  &[(8,1,-168.9532813_f64),(9,1,0.01349188119),(10,0,-0.005575910157_f64)]),
    // 5 GREEN micro_next_h1_green_35343
    (true,  &[(0,1,-2.547097817_f64),(11,1,0.001795740443),(4,2,5.0)]),
    // 6 GREEN micro_next_h1_green_33956
    (true,  &[(8,1,-145.0194062_f64),(9,1,0.01349188119),(12,0,29.60116771)]),
    // 7 GREEN micro_next_h1_green_33974
    (true,  &[(8,1,-145.0194062_f64),(9,1,0.01349188119),(13,1,8.848352749e-05)]),
    // 8 GREEN micro_next_h1_green_4536
    (true,  &[(3,1,3.592157413),(4,2,5.0),(8,1,-145.0194062_f64)]),
    // 9 GREEN micro_next_h1_green_7317
    (true,  &[(3,1,1.679463493),(15,0,0.008140445126),(16,1,-0.00345019142_f64)]),
    // 10 GREEN micro_next_h1_green_26010
    (true,  &[(0,1,-2.346311048_f64),(6,0,0.9656401664),(17,0,-0.01042349892_f64)]),
    // 11 GREEN micro_next_h1_green_38132
    (true,  &[(5,1,-0.01125914436_f64),(18,1,0.002359360549),(19,0,28.1055143)]),
    // 12 GREEN micro_next_h1_green_34351
    (true,  &[(5,1,-0.01125914436_f64),(6,0,0.9656401664),(13,1,8.429477069e-08)]),
    // 13 GREEN micro_next_h1_green_5668
    (true,  &[(14,1,2.359680944),(11,1,0.001795740443),(5,1,-0.07487622772_f64)]),
    // 14 GREEN micro_next_h1_green_14324
    (true,  &[(8,1,-145.0194062_f64),(1,1,2.645309807e-05),(9,0,5.001062838e-05)]),
    // 15 GREEN micro_next_h1_green_4575
    (true,  &[(3,1,3.592157413),(4,2,5.0),(21,1,18.94457115)]),
    // 16 GREEN micro_next_h1_green_21215
    (true,  &[(3,1,1.679463493),(8,1,-145.0194062_f64),(11,1,0.001522731461)]),
    // 17 RED micro_next_h1_red_2328
    (false, &[(5,0,1.202478662),(22,1,1.758262671),(23,1,0.7249823529)]),
    // 18 GREEN micro_next_h1_green_1194
    (true,  &[(5,1,0.04489019383),(6,0,0.9987638412),(20,0,-201.1674785_f64)]),
];

// ── Stratégie ─────────────────────────────────────────────────────────────────

pub struct BtcRules18 {
    buffer: VecDeque<Candle>,
    min_votes: u32,
    rsi7: RsiState,
    rsi8: RsiState,
    atr14: AtrState,
    macd: MacdState,
    last_votes: (u32, u32),
}

impl BtcRules18 {
    pub fn new(min_votes: u32) -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_WINDOW + 1),
            min_votes,
            rsi7: RsiState::new(7),
            rsi8: RsiState::new(8),
            atr14: AtrState::new(14),
            macd: MacdState::new(),
            last_votes: (0, 0),
        }
    }

    fn feed(&mut self, candle: &Candle) {
        self.rsi7.update(candle.close);
        self.rsi8.update(candle.close);
        self.atr14.update(candle);
        self.macd.update(candle.close);
        self.buffer.push_back(candle.clone());
        if self.buffer.len() > MAX_WINDOW {
            self.buffer.pop_front();
        }
    }

    fn vote(&mut self) -> (u32, u32) {
        let feats = compute_feats(
            &self.buffer,
            &self.rsi7, &self.rsi8,
            &self.atr14,
            &self.macd,
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

impl Strategy for BtcRules18 {
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
