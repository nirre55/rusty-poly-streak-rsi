use std::collections::VecDeque;
use chrono::{Datelike, Timelike};
use tracing::debug;

use crate::binance::Candle;
use crate::strategy::{Prediction, Signal, Strategy};

const MAX_WINDOW: usize = 145;
const STRATEGY_NAME: &str = "eth_5m_rules_25_min_votes_1";

// ── RSI Wilder ────────────────────────────────────────────────────────────────

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

// ── ATR ───────────────────────────────────────────────────────────────────────

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

fn williams_r(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n { return None; }
    let highest = buf.iter().rev().take(n).map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
    let lowest  = buf.iter().rev().take(n).map(|c| c.low).fold(f64::INFINITY, f64::min);
    let close = buf.back()?.close;
    let range = highest - lowest;
    if range == 0.0 { return Some(-50.0); }
    Some((highest - close) / range * -100.0)
}

// ── Features ─────────────────────────────────────────────────────────────────
// 0=stoch_k12, 1=stoch_k24, 2=stoch_k72,
// 3=close_z24, 4=close_z48,
// 5=mfi8, 6=mfi14, 7=mfi21,
// 8=atr14_pct, 9=atr72_pct,
// 10=bb_pctb,
// 11=cci12, 12=cci24,
// 13=rsi7, 14=rsi8, 15=rsi21,
// 16=body_abs_pct,
// 17=donch_low72, 18=donch_low144,
// 19=body_sum6, 20=body_sum12,
// 21=hour, 22=lower_wick_body,
// 23=ret12, 24=ret72,
// 25=volume_z96, 26=weekday,
// 27=body_ratio, 28=donch_high12,
// 29=lower_wick, 30=upper_wick_body,
// 31=williams_r12

struct Feats {
    f: [Option<f64>; 32],
}

impl Feats {
    fn get(&self, id: u8) -> Option<f64> { self.f[id as usize] }
}

fn compute_feats(
    buf: &VecDeque<Candle>,
    rsi7: &RsiState, rsi8: &RsiState, rsi21: &RsiState,
    atr14: &AtrState, atr72: &AtrState,
) -> Feats {
    let cur = match buf.back() {
        Some(c) => c,
        None => return Feats { f: [None; 32] },
    };
    let close = cur.close;
    let hour    = cur.close_time.hour() as f64;
    let weekday = cur.close_time.weekday().num_days_from_monday() as f64;
    let lower_wick = if close != 0.0 { (cur.open.min(cur.close) - cur.low) / close } else { 0.0 };
    let body = (cur.close - cur.open).abs();
    let body_abs_pct   = if close != 0.0 { Some(body / close) } else { None };
    let lower_wick_body = if body != 0.0 { Some((cur.open.min(cur.close) - cur.low) / body) } else { None };
    let upper_wick_body = if body != 0.0 { Some((cur.high - cur.open.max(cur.close)) / body) } else { None };
    let body_ratio = { let r = cur.high - cur.low; if r != 0.0 { Some(body / r) } else { None } };

    let mut f: [Option<f64>; 32] = [None; 32];
    f[0]  = stoch_k(buf, 12, close);
    f[1]  = stoch_k(buf, 24, close);
    f[2]  = stoch_k(buf, 72, close);
    f[3]  = close_z(buf, 24);
    f[4]  = close_z(buf, 48);
    f[5]  = mfi(buf, 8);
    f[6]  = mfi(buf, 14);
    f[7]  = mfi(buf, 21);
    f[8]  = atr14.pct(close);
    f[9]  = atr72.pct(close);
    f[10] = bb_pctb(buf);
    f[11] = cci(buf, 12);
    f[12] = cci(buf, 24);
    f[13] = rsi7.get();
    f[14] = rsi8.get();
    f[15] = rsi21.get();
    f[16] = body_abs_pct;
    f[17] = donch_low(buf, 72, close);
    f[18] = donch_low(buf, 144, close);
    f[19] = body_sum(buf, 6);
    f[20] = body_sum(buf, 12);
    f[21] = Some(hour);
    f[22] = lower_wick_body;
    f[23] = ret_n(buf, 12);
    f[24] = ret_n(buf, 72);
    f[25] = vol_z(buf, 96);
    f[26] = Some(weekday);
    f[27] = body_ratio;
    f[28] = donch_high(buf, 12, close);
    f[29] = Some(lower_wick);
    f[30] = upper_wick_body;
    f[31] = williams_r(buf, 12);
    Feats { f }
}

// ── Règles (25) ───────────────────────────────────────────────────────────────
// (green: bool, [(feat_id, cmp: 0=GE 1=LE 2=EQ, threshold); 3])

type Rule = (bool, [(u8, u8, f64); 3]);

fn cmp_ok(val: f64, op: u8, thr: f64) -> bool {
    match op {
        0 => val >= thr,
        1 => val <= thr,
        _ => (val - thr).abs() < 1e-9,
    }
}

fn rule_fires(feats: &Feats, rule: &Rule) -> Option<bool> {
    for &(id, op, thr) in &rule.1 {
        let v = feats.get(id)?;
        if !cmp_ok(v, op, thr) { return None; }
    }
    Some(rule.0)
}

static RULES: &[Rule] = &[
    // 1 green_348: stoch_k24<=0.5443 | hour==5 | rsi21<=39.11
    (true,  [(1,1,0.5443385043),(21,2,5.0),(15,1,39.11398072)]),
    // 2 red_509: close_z24>=3.0829 | weekday==3 | mfi21<=66.52
    (false, [(3,0,3.082851148),(26,2,3.0),(7,1,66.52204642)]),
    // 3 green_219: donch_low72<=0.000570 | bb_pctb<=-0.2409 | close_z48>=-2.697
    (true,  [(17,1,0.0005704541149),(10,1,-0.24090522),(4,0,-2.696757558)]),
    // 4 green_374: stoch_k12<=2.3256 | bb_pctb<=-0.2409 | body_abs_pct<=0.003474
    (true,  [(0,1,2.325581395),(10,1,-0.24090522),(16,1,0.003473998504)]),
    // 5 green_644: donch_low72<=0.000570 | cci12<=-213.82 | stoch_k72>=2.887
    (true,  [(17,1,0.0005704541149),(11,1,-213.8206725),(2,0,2.887028121)]),
    // 6 red_591: rsi8>=84.02 | atr72_pct<=0.000924 | lower_wick_body>=0.01798
    (false, [(14,0,84.02165584),(9,1,0.0009239513225),(22,0,0.01797752809)]),
    // 7 green_723: stoch_k24<=5.1326 | ret72>=0.02600 | rsi7<=24.46
    (true,  [(1,1,5.132606156),(24,0,0.02599541236),(13,1,24.46140344)]),
    // 8 green_1030: donch_low144<=0.000671 | bb_pctb<=-0.2409 | volume_z96<=2.913
    (true,  [(18,1,0.0006709289818),(10,1,-0.24090522),(25,1,2.912906413)]),
    // 9 green_220: donch_low72<=0.000570 | bb_pctb<=-0.2409 | body_sum6>=-0.005522
    (true,  [(17,1,0.0005704541149),(10,1,-0.24090522),(19,0,-0.005522189783)]),
    // 10 red_296: bb_pctb>=1.2423 | weekday==3 | volume_z96<=2.0985
    (false, [(10,0,1.242274122),(26,2,3.0),(25,1,2.098463339)]),
    // 11 red_668: rsi8>=80.23 | close_z48<=1.4568 | mfi14<=73.71
    (false, [(14,0,80.23066448),(4,1,1.456773235),(6,1,73.70646328)]),
    // 12 green_52: mfi8<=7.655 | body_abs_pct>=0.01207 | stoch_k72<=10.167
    (true,  [(5,1,7.65525868),(16,0,0.01206610733),(2,1,10.166951)]),
    // 13 red_262: close_z48>=3.4299 | body_sum12<=0.005753 | rsi21>=67.95
    (false, [(4,0,3.429940156),(20,1,0.005753340807),(15,0,67.95174536)]),
    // 14 red_854: rsi8>=80.23 | atr14_pct<=0.000953 | mfi21>=78.28
    (false, [(14,0,80.23066448),(8,1,0.000953452966),(7,0,78.27879154)]),
    // 15 green_920: donch_high12<=-0.03797 | mfi21<=13.833 | close_z48<=-3.064
    (true,  [(28,1,-0.03797357864),(7,1,13.8331558),(4,1,-3.063615586)]),
    // 16 green_258: cci12<=-243.49 | atr14_pct<=0.001278 | mfi21<=33.23
    (true,  [(11,1,-243.4867158),(8,1,0.001277921698),(7,1,33.22794653)]),
    // 17 green_518: stoch_k12<=2.3256 | hour==11 | williams_r12>=-99.314
    (true,  [(0,1,2.325581395),(21,2,11.0),(31,0,-99.31370042)]),
    // 18 green_1157: cci12<=-243.49 | atr72_pct<=0.001412 | body_abs_pct>=0.003474
    (true,  [(11,1,-243.4867158),(9,1,0.001411663091),(16,0,0.003473998504)]),
    // 19 green_187: mfi8<=13.481 | body_abs_pct>=0.01207 | atr72_pct<=0.006464
    (true,  [(5,1,13.48098558),(16,0,0.01206610733),(9,1,0.00646353357)]),
    // 20 green_215: donch_low72<=0.000570 | upper_wick_body>=5.118 | lower_wick>=1.61e-5
    (true,  [(17,1,0.0005704541149),(30,0,5.117647059),(29,0,1.612549971e-05)]),
    // 21 green_1215: stoch_k12<=0.6863 | close_z24<=-2.809 | body_abs_pct<=0.003474
    (true,  [(0,1,0.6862995766),(3,1,-2.808854839),(16,1,0.003473998504)]),
    // 22 green_533: stoch_k72<=6.987 | body_ratio<=0.03306 | cci12>=-89.88
    (true,  [(2,1,6.986747793),(27,1,0.03305785124),(11,0,-89.88475125)]),
    // 23 green_861: ret12<=-0.03224 | stoch_k12<=3.7793 | cci24<=-175.75
    (true,  [(23,1,-0.03224343338),(0,1,3.779328959),(12,1,-175.7548746)]),
    // 24 green_556: stoch_k24<=1.6446 | bb_pctb<=-0.2409 | lower_wick_body>=0.01798
    (true,  [(1,1,1.644587669),(10,1,-0.24090522),(22,0,0.01797752809)]),
    // 25 green_99: cci12<=-243.49 | atr14_pct<=0.001073 | close_z24<=-2.519
    (true,  [(11,1,-243.4867158),(8,1,0.001072510421),(3,1,-2.519107999)]),
];

// ── Stratégie ─────────────────────────────────────────────────────────────────

pub struct EthRules25 {
    buffer: VecDeque<Candle>,
    min_votes: u32,
    rsi7: RsiState,
    rsi8: RsiState,
    rsi21: RsiState,
    atr14: AtrState,
    atr72: AtrState,
    last_votes: (u32, u32),
}

impl EthRules25 {
    pub fn new(min_votes: u32) -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_WINDOW + 1),
            min_votes,
            rsi7:  RsiState::new(7),
            rsi8:  RsiState::new(8),
            rsi21: RsiState::new(21),
            atr14: AtrState::new(14),
            atr72: AtrState::new(72),
            last_votes: (0, 0),
        }
    }

    fn feed(&mut self, candle: &Candle) {
        self.rsi7.update(candle.close);
        self.rsi8.update(candle.close);
        self.rsi21.update(candle.close);
        self.atr14.update(candle);
        self.atr72.update(candle);
        self.buffer.push_back(candle.clone());
        if self.buffer.len() > MAX_WINDOW {
            self.buffer.pop_front();
        }
    }

    fn vote(&mut self) -> (u32, u32) {
        let feats = compute_feats(
            &self.buffer,
            &self.rsi7, &self.rsi8, &self.rsi21,
            &self.atr14, &self.atr72,
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

impl Strategy for EthRules25 {
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

    fn current_rsi(&self) -> Option<f64> { self.rsi8.get() }
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
