use std::{
    collections::LinkedList,
    time::{Duration, Instant},
};

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
        let warmup = spin(self.config.warmup_duration, 1, &mut setup(), &mut workload);
        std::thread::sleep(self.config.cool_down_duration);
        spin(
            self.config.measuring_duration,
            warmup.iterations,
            &mut setup(),
            &mut workload,
        )
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
    spin_env: &mut T,
    workload: &mut impl FnMut(&mut T) -> BenchIterControl,
) -> BenchIterStats {
    let start = Instant::now();
    let mut iterations = 0;
    let mut early_break = false;
    loop {
        let duration = start.elapsed();
        let enough_duration = at_least_for <= duration;
        if enough_duration || early_break {
            return BenchIterStats {
                iterations,
                duration,
            };
        }
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
}
