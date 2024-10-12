use std::{collections::LinkedList, time::Duration};

use num_traits::Float;

use crate::stopwatch::ElapsedStopwatch;

#[derive(Debug)]
pub struct HeapRandomizer {
    list: LinkedList<usize>,
}
impl HeapRandomizer {
    pub fn new() -> Self {
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
    pub fn new(config: BencherConfig) -> Self {
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
        let batch_running = elapsed.stopwatch_mut().start();
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
