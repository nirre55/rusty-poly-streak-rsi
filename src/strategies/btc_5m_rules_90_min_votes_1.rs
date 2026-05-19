use chrono::{Datelike, Timelike};
use std::collections::VecDeque;
use tracing::debug;

use crate::binance::Candle;
use crate::strategies::indicators::{AtrState, MacdState, RsiState};
use crate::strategy::{Prediction, Signal, Strategy};

const MAX_WINDOW: usize = 145;
const STRATEGY_NAME: &str = "btc_5m_rules_90_min_votes_1";

// RSI Wilder

fn fmean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn fstd_s(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = fmean(v);
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

fn close_z(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let v: Vec<f64> = buf.iter().rev().take(n).map(|c| c.close).collect();
    let s = fstd_s(&v);
    Some(if s == 0.0 {
        0.0
    } else {
        (v[0] - fmean(&v)) / s
    })
}

fn vol_z(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let v: Vec<f64> = buf.iter().rev().take(n).map(|c| c.volume).collect();
    let s = fstd_s(&v);
    Some(if s == 0.0 {
        0.0
    } else {
        (v[0] - fmean(&v)) / s
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
        return None;
    }
    Some(close / min_low - 1.0)
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
        return None;
    }
    Some(close / max_high - 1.0)
}

fn bb_pctb(buf: &VecDeque<Candle>) -> Option<f64> {
    if buf.len() < 20 {
        return None;
    }
    let v: Vec<f64> = buf.iter().rev().take(20).map(|c| c.close).collect();
    let m = fmean(&v);
    let s = fstd_s(&v);
    if s == 0.0 {
        return Some(0.5);
    }
    let upper = m + 2.0 * s;
    let lower = m - 2.0 * s;
    let band = upper - lower;
    if band == 0.0 {
        return Some(0.5);
    }
    Some((v[0] - lower) / band)
}

fn body_sum(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let s = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| {
            if c.close != 0.0 {
                (c.close - c.open) / c.close
            } else {
                0.0
            }
        })
        .sum::<f64>();
    Some(s)
}

fn stoch_k(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let min_l = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min);
    let max_h = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let range = max_h - min_l;
    Some(if range == 0.0 {
        50.0
    } else {
        (close - min_l) / range * 100.0
    })
}

fn ret_n(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    let needed = n + 1;
    if buf.len() < needed {
        return None;
    }
    let cur = buf[buf.len() - 1].close;
    let past = buf[buf.len() - 1 - n].close;
    if past == 0.0 {
        None
    } else {
        Some(cur / past - 1.0)
    }
}

fn dist_sma(buf: &VecDeque<Candle>, n: usize, close: f64) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let sma = fmean(
        &buf.iter()
            .rev()
            .take(n)
            .map(|c| c.close)
            .collect::<Vec<_>>(),
    );
    if sma == 0.0 {
        None
    } else {
        Some(close / sma - 1.0)
    }
}

fn cci(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let tps: Vec<f64> = buf
        .iter()
        .rev()
        .take(n)
        .map(|c| (c.high + c.low + c.close) / 3.0)
        .collect();
    let m = fmean(&tps);
    let md = tps.iter().map(|x| (x - m).abs()).sum::<f64>() / n as f64;
    if md == 0.0 {
        return Some(0.0);
    }
    Some((tps[0] - m) / (0.015 * md))
}

fn mfi(buf: &VecDeque<Candle>, n: usize) -> Option<f64> {
    if buf.len() < n + 1 {
        return None;
    }
    let start = buf.len() - n - 1;
    let (mut pos, mut neg) = (0.0f64, 0.0f64);
    for i in (start + 1)..buf.len() {
        let prev_tp = (buf[i - 1].high + buf[i - 1].low + buf[i - 1].close) / 3.0;
        let curr_tp = (buf[i].high + buf[i].low + buf[i].close) / 3.0;
        let rmf = curr_tp * buf[i].volume;
        if curr_tp > prev_tp {
            pos += rmf;
        } else if curr_tp < prev_tp {
            neg += rmf;
        }
    }
    Some(if neg == 0.0 {
        if pos == 0.0 {
            50.0
        } else {
            100.0
        }
    } else {
        100.0 - 100.0 / (1.0 + pos / neg)
    })
}

fn count_color(buf: &VecDeque<Candle>, n: usize, green: bool) -> Option<f64> {
    if buf.len() < n {
        return None;
    }
    let c = buf
        .iter()
        .rev()
        .take(n)
        .filter(|c| {
            if green {
                c.close > c.open
            } else {
                c.close < c.open
            }
        })
        .count();
    Some(c as f64)
}

// Features
// 0=close_z24, 1=close_z48, 2=donch_low72, 3=donch_low144, 4=donch_high12,
// 5=donch_high72, 6=bb_pctb, 7=atr14_pct, 8=atr72_pct, 9=body_sum6,
// 10=body_sum12, 11=hour, 12=weekday, 13=rsi8, 14=rsi14, 15=rsi21,
// 16=stoch_k12, 17=stoch_k24, 18=ret12, 19=ret24, 20=ret72,
// 21=dist_sma24, 22=cci12, 23=cci24, 24=cci72, 25=lower_wick,
// 26=upper_wick, 27=mfi8, 28=mfi14, 29=mfi21, 30=volume_z96,
// 31=macd_hist_pct, 32=green_count6, 33=red_count6

struct Feats {
    f: [Option<f64>; 34],
}

impl Feats {
    fn get(&self, id: u8) -> Option<f64> {
        self.f[id as usize]
    }
}

fn compute_feats(
    buf: &VecDeque<Candle>,
    rsi8: &RsiState,
    rsi14: &RsiState,
    rsi21: &RsiState,
    atr14: &AtrState,
    atr72: &AtrState,
    macd: &MacdState,
) -> Feats {
    let cur = match buf.back() {
        Some(c) => c,
        None => return Feats { f: [None; 34] },
    };
    let close = cur.close;
    let hour = cur.close_time.hour() as f64;
    let weekday = cur.close_time.weekday().num_days_from_monday() as f64;
    let lower_wick = (cur.open.min(cur.close) - cur.low) / close;
    let upper_wick = (cur.high - cur.open.max(cur.close)) / close;

    let mut f: [Option<f64>; 34] = [None; 34];
    f[0] = close_z(buf, 24);
    f[1] = close_z(buf, 48);
    f[2] = donch_low(buf, 72, close);
    f[3] = donch_low(buf, 144, close);
    f[4] = donch_high(buf, 12, close);
    f[5] = donch_high(buf, 72, close);
    f[6] = bb_pctb(buf);
    f[7] = atr14.pct(close);
    f[8] = atr72.pct(close);
    f[9] = body_sum(buf, 6);
    f[10] = body_sum(buf, 12);
    f[11] = Some(hour);
    f[12] = Some(weekday);
    f[13] = rsi8.get();
    f[14] = rsi14.get();
    f[15] = rsi21.get();
    f[16] = stoch_k(buf, 12, close);
    f[17] = stoch_k(buf, 24, close);
    f[18] = ret_n(buf, 12);
    f[19] = ret_n(buf, 24);
    f[20] = ret_n(buf, 72);
    f[21] = dist_sma(buf, 24, close);
    f[22] = cci(buf, 12);
    f[23] = cci(buf, 24);
    f[24] = cci(buf, 72);
    f[25] = Some(lower_wick);
    f[26] = Some(upper_wick);
    f[27] = mfi(buf, 8);
    f[28] = mfi(buf, 14);
    f[29] = mfi(buf, 21);
    f[30] = vol_z(buf, 96);
    f[31] = macd.hist_pct(close);
    f[32] = count_color(buf, 6, true);
    f[33] = count_color(buf, 6, false);
    Feats { f }
}

// Rules (90)
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
        if !cmp_ok(v, op, thr) {
            return None;
        }
    }
    Some(rule.0)
}

static RULES: &[Rule] = &[
    // 1 RED
    (
        false,
        [
            (16, 0, 98.87542722),
            (18, 0, 0.02486548257),
            (25, 1, 0.001638794532),
        ],
    ),
    // 2 GREEN
    (
        true,
        [(6, 1, -0.107140425), (11, 2, 13.0), (30, 1, 0.7579850134)],
    ),
    // 3 GREEN
    (
        true,
        [
            (22, 1, -239.1832969),
            (8, 1, 0.0006406964493),
            (13, 0, 16.98439155),
        ],
    ),
    // 4 GREEN
    (
        true,
        [
            (3, 1, 0.001457840018),
            (9, 1, -0.01817602056),
            (30, 0, 3.587853962),
        ],
    ),
    // 5 GREEN
    (
        true,
        [
            (1, 1, -2.44769017),
            (8, 1, 0.0006406964493),
            (10, 1, -0.004124811926),
        ],
    ),
    // 6 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (8, 1, 0.0006406964493),
            (17, 0, 10.74126157),
        ],
    ),
    // 7 GREEN
    (
        true,
        [
            (15, 1, 33.28245704),
            (19, 0, -0.005787322997),
            (21, 1, -0.005476785129),
        ],
    ),
    // 8 GREEN
    (
        true,
        [
            (6, 1, -0.2340435963),
            (11, 2, 13.0),
            (21, 0, -0.007134307442),
        ],
    ),
    // 9 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (8, 1, 0.0006406964493),
            (6, 0, -0.2340435963),
        ],
    ),
    // 10 GREEN
    (
        true,
        [
            (2, 1, 0.0008266398454),
            (10, 1, -0.01970911953),
            (30, 0, 2.919374564),
        ],
    ),
    // 11 GREEN
    (
        true,
        [
            (3, 1, 0.0006404179203),
            (11, 2, 14.0),
            (20, 1, -0.01475596055),
        ],
    ),
    // 12 RED
    (
        false,
        [
            (4, 0, -0.000239825686),
            (18, 0, 0.01939351557),
            (25, 1, 0.001638794532),
        ],
    ),
    // 13 GREEN
    (
        true,
        [
            (22, 1, -239.1832969),
            (0, 0, -2.058231615),
            (25, 1, 0.001092315747),
        ],
    ),
    // 14 RED
    (
        false,
        [
            (31, 0, 0.001865948161),
            (16, 0, 97.6614989),
            (0, 0, 2.292740233),
        ],
    ),
    // 15 GREEN
    (
        true,
        [
            (17, 1, 2.898900732),
            (4, 1, -0.02994953395),
            (27, 1, 14.51065973),
        ],
    ),
    // 16 GREEN
    (
        true,
        [
            (6, 1, -0.107140425),
            (8, 1, 0.0004654084234),
            (7, 0, 0.0002988690878),
        ],
    ),
    // 17 GREEN
    (
        true,
        [(6, 1, -0.173645982), (11, 2, 13.0), (27, 0, 25.84167143)],
    ),
    // 18 GREEN
    (
        true,
        [
            (22, 1, -209.9115581),
            (8, 1, 0.0007461976822),
            (7, 0, 0.0006619818413),
        ],
    ),
    // 19 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (8, 1, 0.0007461976822),
            (24, 1, -130.1004463),
        ],
    ),
    // 20 GREEN
    (
        true,
        [
            (22, 1, -239.1832969),
            (8, 1, 0.0007461976822),
            (17, 0, 7.498316732),
        ],
    ),
    // 21 GREEN
    (
        true,
        [
            (0, 1, -2.487372513),
            (8, 1, 0.0004654084234),
            (23, 0, -192.094143),
        ],
    ),
    // 22 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (7, 1, 0.0007499679689),
            (1, 1, -3.032681751),
        ],
    ),
    // 23 RED
    (
        false,
        [
            (16, 0, 95.36043284),
            (31, 0, 0.002366750995),
            (0, 0, 2.046146229),
        ],
    ),
    // 24 RED
    (
        false,
        [
            (31, 0, 0.001865948161),
            (16, 0, 95.36043284),
            (29, 0, 73.95404425),
        ],
    ),
    // 25 RED
    (
        false,
        [
            (27, 0, 94.13387702),
            (4, 0, -0.0001214530509),
            (17, 1, 99.45945562),
        ],
    ),
    // 26 RED
    (
        false,
        [
            (31, 0, 0.001865948161),
            (16, 0, 98.87542722),
            (20, 1, 0.03681466689),
        ],
    ),
    // 27 GREEN
    (
        true,
        [
            (6, 1, -0.2340435963),
            (21, 0, -0.004431553752),
            (24, 1, -153.295298),
        ],
    ),
    // 28 GREEN
    (
        true,
        [
            (6, 1, -0.2340435963),
            (21, 0, -0.004431553752),
            (14, 1, 27.52356397),
        ],
    ),
    // 29 GREEN
    (
        true,
        [
            (6, 1, -0.107140425),
            (8, 1, 0.0006406964493),
            (2, 0, 0.001652921292),
        ],
    ),
    // 30 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (8, 1, 0.0006406964493),
            (27, 1, 18.35512826),
        ],
    ),
    // 31 RED
    (
        false,
        [
            (16, 0, 95.36043284),
            (10, 0, 0.01911639022),
            (5, 1, -0.001380130085),
        ],
    ),
    // 32 GREEN
    (
        true,
        [
            (6, 1, -0.2340435963),
            (21, 0, -0.004431553752),
            (28, 1, 27.45320025),
        ],
    ),
    // 33 GREEN
    (
        true,
        [(0, 1, -2.774117242), (8, 1, 0.0006406964493), (32, 0, 2.0)],
    ),
    // 34 GREEN
    (
        true,
        [
            (1, 1, -2.668322486),
            (8, 1, 0.0004654084234),
            (28, 1, 31.37307417),
        ],
    ),
    // 35 GREEN
    (
        true,
        [
            (3, 1, 0.0006404179203),
            (4, 1, -0.02387268949),
            (32, 1, 1.0),
        ],
    ),
    // 36 RED
    (false, [(13, 0, 79.78754453), (11, 2, 21.0), (33, 0, 2.0)]),
    // 37 RED
    (
        false,
        [
            (16, 0, 95.36043284),
            (10, 0, 0.01911639022),
            (24, 0, 301.1917591),
        ],
    ),
    // 38 GREEN
    (
        true,
        [
            (2, 1, 0.0008266398454),
            (19, 1, -0.03454574655),
            (26, 0, 0.001054938882),
        ],
    ),
    // 39 RED
    (
        false,
        [
            (16, 0, 95.36043284),
            (7, 0, 0.005489115066),
            (8, 1, 0.003517992609),
        ],
    ),
    // 40 RED
    (
        false,
        [
            (31, 0, 0.001865948161),
            (16, 0, 95.36043284),
            (17, 1, 96.70846245),
        ],
    ),
    // 41 GREEN
    (
        true,
        [
            (6, 1, -0.107140425),
            (8, 1, 0.0006406964493),
            (17, 0, 4.53517561),
        ],
    ),
    // 42 GREEN
    (
        true,
        [(17, 1, 1.091431392), (9, 1, -0.01392502169), (32, 1, 1.0)],
    ),
    // 43 RED
    (
        false,
        [
            (16, 0, 95.36043284),
            (9, 0, 0.01753783257),
            (25, 1, 0.001638794532),
        ],
    ),
    // 44 GREEN
    (
        true,
        [(0, 1, -2.487372513), (11, 2, 13.0), (30, 1, 1.219114982)],
    ),
    // 45 RED
    (
        false,
        [
            (16, 0, 95.36043284),
            (10, 0, 0.02432417868),
            (25, 1, 0.001638794532),
        ],
    ),
    // 46 RED
    (
        false,
        [(16, 0, 95.36043284), (18, 0, 0.01939351557), (12, 2, 4.0)],
    ),
    // 47 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (21, 0, -0.004431553752),
            (1, 1, -3.385888687),
        ],
    ),
    // 48 GREEN
    (
        true,
        [
            (6, 1, -0.06443813881),
            (8, 1, 0.0004654084234),
            (15, 0, 39.3931585),
        ],
    ),
    // 49 GREEN
    (
        true,
        [(0, 1, -2.774117242), (2, 1, 0.001228070749), (11, 2, 6.0)],
    ),
    // 50 GREEN
    (
        true,
        [
            (3, 1, 0.001457840018),
            (1, 1, -3.385888687),
            (4, 0, -0.007190350995),
        ],
    ),
    // 51 RED
    (
        false,
        [
            (9, 0, 0.008390907843),
            (4, 0, -0.0003773777571),
            (25, 1, 0.0),
        ],
    ),
    // 52 GREEN
    (
        true,
        [
            (3, 1, 0.001457840018),
            (1, 1, -3.385888687),
            (21, 1, -0.0156322462),
        ],
    ),
    // 53 GREEN
    (
        true,
        [
            (0, 1, -2.304024546),
            (11, 2, 22.0),
            (26, 1, 9.223945209e-08),
        ],
    ),
    // 54 RED
    (
        false,
        [(0, 0, 2.783849801), (11, 2, 9.0), (1, 1, 3.429563387)],
    ),
    // 55 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (10, 0, -0.005758468918),
            (9, 1, -0.004965357046),
        ],
    ),
    // 56 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (10, 0, -0.005758468918),
            (11, 2, 13.0),
        ],
    ),
    // 57 RED
    (
        false,
        [
            (4, 0, -0.000509590257),
            (2, 1, 0.0005943956918),
            (25, 1, 3.702010378e-07),
        ],
    ),
    // 58 GREEN
    (
        true,
        [
            (13, 1, 31.93496681),
            (20, 0, 0.0242546669),
            (10, 0, -0.007039969776),
        ],
    ),
    // 59 GREEN
    (
        true,
        [
            (2, 1, 0.0008266398454),
            (21, 1, -0.0156322462),
            (20, 0, -0.02315688552),
        ],
    ),
    // 60 RED
    (
        false,
        [(13, 0, 79.78754453), (11, 2, 21.0), (14, 1, 73.34429789)],
    ),
    // 61 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (18, 0, -0.005712097907),
            (15, 1, 31.37459303),
        ],
    ),
    // 62 GREEN
    (
        true,
        [
            (15, 1, 33.28245704),
            (19, 0, -0.005787322997),
            (6, 1, -0.173645982),
        ],
    ),
    // 63 GREEN
    (
        true,
        [
            (6, 1, -0.2340435963),
            (21, 0, -0.004431553752),
            (9, 1, -0.004083019831),
        ],
    ),
    // 64 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (15, 0, 31.37459303),
            (2, 1, 0.0001481980796),
        ],
    ),
    // 65 RED
    (
        false,
        [(0, 0, 3.068414947), (12, 2, 5.0), (1, 0, 3.429563387)],
    ),
    // 66 RED
    (
        false,
        [(5, 0, -0.000212151614), (11, 2, 12.0), (6, 1, 1.061948757)],
    ),
    // 67 RED
    (
        false,
        [(9, 0, 0.01364096683), (25, 1, 0.0), (26, 1, 0.001347937908)],
    ),
    // 68 GREEN
    (
        true,
        [(0, 1, -2.487372513), (11, 2, 11.0), (9, 0, -0.004965357046)],
    ),
    // 69 GREEN
    (true, [(6, 1, -0.06443813881), (11, 2, 13.0), (12, 2, 5.0)]),
    // 70 GREEN
    (
        true,
        [
            (13, 1, 23.24758078),
            (3, 0, 0.03033879208),
            (0, 1, -2.304024546),
        ],
    ),
    // 71 GREEN
    (
        true,
        [
            (13, 1, 31.93496681),
            (2, 0, 0.03468189691),
            (5, 0, -0.02252162313),
        ],
    ),
    // 72 GREEN
    (
        true,
        [
            (0, 1, -2.058231615),
            (2, 0, 0.02558085724),
            (5, 0, -0.02252162313),
        ],
    ),
    // 73 GREEN
    (
        true,
        [(6, 1, -0.2340435963), (2, 1, 0.001228070749), (12, 2, 1.0)],
    ),
    // 74 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (9, 0, -0.002956606273),
            (15, 1, 35.7929937),
        ],
    ),
    // 75 RED
    (
        false,
        [(0, 0, 3.068414947), (12, 2, 5.0), (18, 1, 0.004280476703)],
    ),
    // 76 RED
    (
        false,
        [
            (0, 0, 3.068414947),
            (1, 1, 1.903776951),
            (3, 1, 0.03033879208),
        ],
    ),
    // 77 GREEN
    (
        true,
        [(0, 1, -2.487372513), (11, 2, 22.0), (30, 1, 0.7579850134)],
    ),
    // 78 GREEN
    (
        true,
        [
            (6, 1, -0.2340435963),
            (21, 0, -0.004431553752),
            (3, 1, 0.001091384768),
        ],
    ),
    // 79 RED
    (
        false,
        [(13, 0, 79.78754453), (12, 2, 5.0), (15, 1, 64.95549342)],
    ),
    // 80 RED
    (
        false,
        [(0, 0, 2.783849801), (12, 2, 5.0), (5, 1, -0.001004677022)],
    ),
    // 81 GREEN
    (
        true,
        [
            (6, 1, -0.2340435963),
            (14, 0, 35.01130025),
            (21, 1, -0.003142104413),
        ],
    ),
    // 82 GREEN
    (
        true,
        [
            (6, 1, -0.005731108736),
            (2, 0, 0.02923394723),
            (3, 0, 0.04226528076),
        ],
    ),
    // 83 GREEN
    (
        true,
        [
            (6, 1, -0.107140425),
            (11, 2, 22.0),
            (19, 0, -0.005787322997),
        ],
    ),
    // 84 GREEN
    (
        true,
        [
            (0, 1, -2.774117242),
            (10, 0, -0.005758468918),
            (3, 1, 0.001091384768),
        ],
    ),
    // 85 GREEN
    (
        true,
        [(0, 1, -2.487372513), (11, 2, 13.0), (9, 0, -0.004083019831)],
    ),
    // 86 RED
    (
        false,
        [(14, 0, 77.03278368), (11, 2, 11.0), (30, 1, 2.919374564)],
    ),
    // 87 RED
    (
        false,
        [
            (13, 0, 73.82299439),
            (20, 1, -0.01475596055),
            (1, 1, 1.450800747),
        ],
    ),
    // 88 RED
    (
        false,
        [
            (9, 0, 0.01364096683),
            (4, 0, -0.000509590257),
            (19, 1, 0.0173612306),
        ],
    ),
    // 89 RED
    (
        false,
        [
            (0, 0, 2.783849801),
            (1, 1, 1.450800747),
            (18, 0, 0.004280476703),
        ],
    ),
    // 90 GREEN
    (
        true,
        [(6, 1, -0.107140425), (11, 2, 11.0), (3, 1, 0.004305280217)],
    ),
];

// Strategy

pub struct BtcRules90 {
    buffer: VecDeque<Candle>,
    min_votes: u32,
    rsi8: RsiState,
    rsi14: RsiState,
    rsi21: RsiState,
    atr14: AtrState,
    atr72: AtrState,
    macd: MacdState,
    last_votes: (u32, u32),
}

impl BtcRules90 {
    pub fn new(min_votes: u32) -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_WINDOW + 1),
            min_votes,
            rsi8: RsiState::new(8),
            rsi14: RsiState::new(14),
            rsi21: RsiState::new(21),
            atr14: AtrState::new(14),
            atr72: AtrState::new(72),
            macd: MacdState::new(),
            last_votes: (0, 0),
        }
    }

    fn feed(&mut self, candle: &Candle) {
        self.rsi8.update(candle.close);
        self.rsi14.update(candle.close);
        self.rsi21.update(candle.close);
        self.atr14.update(candle);
        self.atr72.update(candle);
        self.macd.update(candle.close);
        self.buffer.push_back(candle.clone());
        if self.buffer.len() > MAX_WINDOW {
            self.buffer.pop_front();
        }
    }

    fn vote(&mut self) -> (u32, u32) {
        let feats = compute_feats(
            &self.buffer,
            &self.rsi8,
            &self.rsi14,
            &self.rsi21,
            &self.atr14,
            &self.atr72,
            &self.macd,
        );
        let (mut gv, mut rv) = (0u32, 0u32);
        for rule in RULES {
            if let Some(green) = rule_fires(&feats, rule) {
                if green {
                    gv += 1;
                } else {
                    rv += 1;
                }
            }
        }
        self.last_votes = (gv, rv);
        (gv, rv)
    }
}

impl Strategy for BtcRules90 {
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

        let (gv, rv) = self.vote();
        let total = gv + rv;

        debug!(
            "[ENSEMBLE] green_votes={} red_votes={} total={} min_votes={}",
            gv, rv, total, self.min_votes
        );

        if total < self.min_votes {
            return None;
        }
        if gv == rv {
            return None;
        }

        let prediction = if gv > rv {
            Prediction::Up
        } else {
            Prediction::Down
        };
        let vote_pct = if gv > rv {
            gv as f64 / total as f64 * 100.0
        } else {
            rv as f64 / total as f64 * 100.0
        };

        Some(Signal {
            prediction,
            signal_candle_close_time: candle.close_time,
            rsi: vote_pct,
            strategy_name: self.name().to_string(),
        })
    }

    fn current_rsi(&self) -> Option<f64> {
        self.rsi14.get()
    }
    fn current_series(&self) -> Option<bool> {
        None
    }
    fn current_atr(&self) -> Option<f64> {
        self.atr14.raw()
    }

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
