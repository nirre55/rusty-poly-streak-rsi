use crate::binance::Candle;

pub(super) struct RsiState {
    period: usize,
    seed: Vec<(f64, f64)>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
    rsi: Option<f64>,
    last_close: Option<f64>,
}

impl RsiState {
    pub(super) fn new(period: usize) -> Self {
        Self {
            period,
            seed: Vec::with_capacity(period),
            avg_gain: None,
            avg_loss: None,
            rsi: None,
            last_close: None,
        }
    }

    pub(super) fn update(&mut self, close: f64) {
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
                let ag = (self.avg_gain.expect("avg_gain initialized") * (p - 1.0) + g) / p;
                let al = (self.avg_loss.expect("avg_loss initialized") * (p - 1.0) + l) / p;
                self.avg_gain = Some(ag);
                self.avg_loss = Some(al);
                self.rsi = Some(rsi_val(ag, al));
            }
        }
        self.last_close = Some(close);
    }

    pub(super) fn get(&self) -> Option<f64> {
        self.rsi
    }
}

fn rsi_val(ag: f64, al: f64) -> f64 {
    if al == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + ag / al)
    }
}

pub(super) struct AtrState {
    period: usize,
    seed: Vec<f64>,
    atr: Option<f64>,
    last_close: Option<f64>,
}

impl AtrState {
    pub(super) fn new(period: usize) -> Self {
        Self {
            period,
            seed: Vec::with_capacity(period),
            atr: None,
            last_close: None,
        }
    }

    pub(super) fn update(&mut self, candle: &Candle) {
        if let Some(prev) = self.last_close {
            let tr = (candle.high - candle.low)
                .max((candle.high - prev).abs())
                .max((candle.low - prev).abs());
            if self.atr.is_none() {
                self.seed.push(tr);
                if self.seed.len() == self.period {
                    self.atr = Some(self.seed.iter().sum::<f64>() / self.period as f64);
                }
            } else {
                let p = self.period as f64;
                self.atr = Some((self.atr.expect("atr initialized") * (p - 1.0) + tr) / p);
            }
        }
        self.last_close = Some(candle.close);
    }

    pub(super) fn pct(&self, close: f64) -> Option<f64> {
        self.atr.map(|atr| atr / close)
    }

    pub(super) fn raw(&self) -> Option<f64> {
        self.atr
    }
}

pub(super) struct MacdState {
    ema12: Option<f64>,
    ema26: Option<f64>,
    signal: Option<f64>,
    hist: Option<f64>,
    n: usize,
}

impl MacdState {
    pub(super) fn new() -> Self {
        Self {
            ema12: None,
            ema26: None,
            signal: None,
            hist: None,
            n: 0,
        }
    }

    pub(super) fn update(&mut self, close: f64) {
        self.n += 1;
        let a12 = 2.0 / 13.0;
        let a26 = 2.0 / 27.0;
        let a9 = 2.0 / 10.0;
        self.ema12 = Some(match self.ema12 {
            None => close,
            Some(e) => e + a12 * (close - e),
        });
        self.ema26 = Some(match self.ema26 {
            None => close,
            Some(e) => e + a26 * (close - e),
        });
        if self.n >= 26 {
            let m = self.ema12.expect("ema12 initialized") - self.ema26.expect("ema26 initialized");
            self.signal = Some(match self.signal {
                None => m,
                Some(s) => s + a9 * (m - s),
            });
            self.hist = Some(m - self.signal.expect("signal initialized"));
        }
    }

    pub(super) fn line_pct(&self, close: f64) -> Option<f64> {
        if self.n < 26 {
            return None;
        }
        Some((self.ema12? - self.ema26?) / close)
    }

    pub(super) fn hist_pct(&self, close: f64) -> Option<f64> {
        self.hist.map(|hist| hist / close)
    }
}
