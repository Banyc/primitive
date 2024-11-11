use core::{num::NonZeroUsize, time::Duration};
use std::collections::LinkedList;

use num_traits::Float;

use crate::{
    ops::float::{NonNegF, PosF, UnitF},
    time::stopwatch::ElapsedStopwatch,
    Clear,
};

#[derive(Debug)]
pub struct HeapRandomizer {
    list: LinkedList<usize>,
}
impl HeapRandomizer {
    pub const fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }

    const DEPTH: usize = 2 << 9;
    pub fn randomize(&mut self) {
        for i in 0..Self::DEPTH {
            self.list.push_back(i);
        }
    }
}
impl Default for HeapRandomizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BencherConfig {
    pub warmup_duration: Duration,
    pub cool_down_duration: Duration,
    pub measuring_duration: Duration,
}
impl Default for BencherConfig {
    fn default() -> Self {
        Self {
            warmup_duration: Duration::from_millis(100),
            cool_down_duration: Duration::from_secs(1),
            measuring_duration: Duration::from_secs(5),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Bencher {
    config: BencherConfig,
}
impl Bencher {
    pub const fn new(config: BencherConfig) -> Self {
        Self { config }
    }

    pub fn iter<T>(
        &self,
        setup: impl Fn() -> T,
        mut workload: impl FnMut(&mut T) -> BenchIterControl,
    ) -> BenchIterStats {
        let warmup = spin(
            self.config.warmup_duration,
            1,
            None,
            &mut setup(),
            &mut workload,
        );
        std::thread::sleep(self.config.cool_down_duration);
        let mut cum_var_secs = CumVar::new(warmup.mean_secs());
        let measuring = spin(
            self.config.measuring_duration,
            warmup.iterations,
            Some(&mut cum_var_secs),
            &mut setup(),
            &mut workload,
        );
        BenchIterStats {
            iterations: measuring.iterations,
            duration: measuring.duration,
            variance_secs: cum_var_secs.get(),
        }
    }
}
#[allow(clippy::derivable_impls)]
impl Default for Bencher {
    fn default() -> Self {
        Self {
            config: BencherConfig::default(),
        }
    }
}
fn spin<T>(
    at_least_for: Duration,
    batch_size: usize,
    mut cum_var_secs: Option<&mut CumVar<f64>>,
    spin_env: &mut T,
    mut workload: impl FnMut(&mut T) -> BenchIterControl,
) -> SpinStats {
    let mut elapsed = ElapsedStopwatch::new(at_least_for);
    let mut iterations = 0;
    let mut early_break = false;
    loop {
        let duration = elapsed.stopwatch().elapsed();
        let enough_duration = at_least_for <= duration;
        if enough_duration || early_break {
            return SpinStats {
                iterations,
                duration,
            };
        }
        let batch_running = elapsed.stopwatch_mut().start_scoped();
        for _ in 0..batch_size {
            let ctrl = workload(spin_env);
            iterations += 1;
            match ctrl {
                BenchIterControl::Continue => (),
                BenchIterControl::Break => {
                    early_break = true;
                    break;
                }
            }
        }
        let batch_elapsed = batch_running.stop();
        if let Some(cum_var) = cum_var_secs.as_deref_mut() {
            cum_var.update(batch_elapsed.as_secs_f64() / batch_size as f64);
        }
    }
}
#[derive(Debug, Clone)]
struct SpinStats {
    pub iterations: usize,
    pub duration: Duration,
}
impl SpinStats {
    pub fn mean_secs(&self) -> f64 {
        self.duration.as_secs_f64() / self.iterations as f64
    }
}
#[derive(Debug, Clone)]
pub enum BenchIterControl {
    Continue,
    Break,
}
#[derive(Debug, Clone)]
pub struct BenchIterStats {
    pub iterations: usize,
    pub duration: Duration,
    pub variance_secs: f64,
}
impl BenchIterStats {
    pub fn mean_secs(&self) -> f64 {
        self.duration.as_secs_f64() / self.iterations as f64
    }
    pub fn standard_deviation_secs(&self) -> f64 {
        self.variance_secs.sqrt()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CumVar<R> {
    sum: R,
    sum_of_squared: R,
    n: R,
    rough_mean: R,
}
impl<R> CumVar<R>
where
    R: Float,
{
    pub fn new(rough_mean: R) -> Self {
        Self {
            sum: R::zero(),
            sum_of_squared: R::zero(),
            n: R::zero(),
            rough_mean,
        }
    }
    pub fn update(&mut self, x: R) {
        let adjusted = x - self.rough_mean;
        self.sum = self.sum + adjusted;
        self.sum_of_squared = self.sum_of_squared + adjusted.powi(2);
        self.n = self.n + R::one();
    }
    pub fn get(&self) -> R {
        let expect_of_squared = self.sum_of_squared / self.n;
        let expect_squared = (self.sum / self.n).powi(2);
        expect_of_squared - expect_squared
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExpMovAvg<R> {
    alpha: R,
    prev: Option<R>,
}
impl<R> ExpMovAvg<R>
where
    R: Float + From<f64>,
{
    pub const fn from_alpha(alpha: R) -> Self {
        Self { prev: None, alpha }
    }
    pub fn from_periods(n: NonZeroUsize) -> Self {
        let alpha = 2. / (1 + n.get()) as f64;
        Self::from_alpha(alpha.into())
    }

    pub const fn get(&self) -> Option<R> {
        self.prev
    }
    pub fn update(&mut self, x: R) {
        let Some(prev) = self.prev else {
            self.prev = Some(x);
            return;
        };
        let new = x * self.alpha;
        let old = prev * (R::one() - self.alpha);
        self.prev = Some(new + old);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExpMovVar<R> {
    mean: ExpMovAvg<R>,
    var: ExpMovAvg<R>,
}
impl<R> ExpMovVar<R>
where
    R: Float + From<f64>,
{
    pub const fn from_alpha(alpha: R) -> Self {
        Self {
            mean: ExpMovAvg::from_alpha(alpha),
            var: ExpMovAvg::from_alpha(alpha),
        }
    }
    pub fn from_periods(n: NonZeroUsize) -> Self {
        Self {
            mean: ExpMovAvg::from_periods(n),
            var: ExpMovAvg::from_periods(n),
        }
    }

    pub fn update(&mut self, x: R) {
        let var = self.mean.get().map(|mean| (x - mean).powi(2));
        self.mean.update(x);
        if let Some(var) = var {
            self.var.update(var);
        }
    }
    pub const fn mean(&self) -> &ExpMovAvg<R> {
        &self.mean
    }
    pub const fn var(&self) -> &ExpMovAvg<R> {
        &self.var
    }
}
#[cfg(test)]
#[test]
fn test_ema() {
    let mut ema = ExpMovVar::from_periods(NonZeroUsize::new(2).unwrap());
    ema.update(2.);
    ema.update(3.);
    ema.update(4.);
    dbg!(ema.mean().get());
    dbg!(ema.var().get().map(|x| x.sqrt()));
    assert!(3. < ema.mean().get().unwrap());
    assert!(ema.mean().get().unwrap() < 4.);
}

#[derive(Debug, Clone, Copy)]
pub struct NearZeroHistogram<const N: usize> {
    buckets: [u64; N],
    count: usize,
    a: f64,
}
impl<const N: usize> NearZeroHistogram<N> {
    #[must_use]
    pub fn new(max_value: PosF<f64>) -> Self {
        let a = (N as f64) / max_value.get().ln_1p();
        Self {
            buckets: [0; N],
            count: 0,
            a,
        }
    }
    pub fn insert(&mut self, value: NonNegF<f64>) {
        self.count += 1;
        let bucket = self.a * value.get().ln_1p();
        let bucket = bucket.round();
        if N as f64 <= bucket {
            return;
        }
        let bucket = bucket as usize;
        if N <= bucket {
            return;
        }
        self.buckets[bucket] += 1;
    }
    #[must_use]
    pub fn quartile(&self, p: UnitF<f64>) -> QuartileResult {
        let Some(n) = self.count.checked_sub(1) else {
            return QuartileResult::NoSamples;
        };
        let i = n as f64 * p.get();
        let mut remaining = i as u64;
        let mut found_bucket = None;
        for (bucket, count) in self.buckets.iter().copied().enumerate() {
            if remaining < count {
                found_bucket = Some(bucket);
                break;
            }
            remaining -= count;
        }
        let Some(found_bucket) = found_bucket else {
            return QuartileResult::OutOfMaxValue;
        };
        let bucket = found_bucket as f64;
        let value = (bucket / self.a).exp_m1();
        QuartileResult::Found(value)
    }
}
impl<const N: usize> Clear for NearZeroHistogram<N> {
    fn clear(&mut self) {
        self.buckets = [0; N];
        self.count = 0;
    }
}
#[derive(Debug, Clone, Copy)]
pub enum QuartileResult {
    NoSamples,
    OutOfMaxValue,
    Found(f64),
}
