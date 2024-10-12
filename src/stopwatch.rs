use std::time::{Duration, Instant};

use crate::Clear;

#[derive(Debug, Clone)]
pub struct Stopwatch {
    elapsed: Duration,
}
impl Stopwatch {
    pub fn new(elapsed: Duration) -> Self {
        Self { elapsed }
    }
    pub fn start(&mut self) -> RunningWatch<'_> {
        RunningWatch {
            stopwatch: self,
            start: Instant::now(),
        }
    }
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}
impl Default for Stopwatch {
    fn default() -> Self {
        Self::new(Duration::ZERO)
    }
}
impl Clear for Stopwatch {
    fn clear(&mut self) {
        self.elapsed = Duration::ZERO;
    }
}

#[derive(Debug)]
pub struct RunningWatch<'a> {
    stopwatch: &'a mut Stopwatch,
    start: Instant,
}
impl RunningWatch<'_> {
    pub fn start(&self) -> Instant {
        self.start
    }
    pub fn stop(mut self) -> Duration {
        self.record_elapsed()
    }
    fn record_elapsed(&mut self) -> Duration {
        let elapsed = self.start.elapsed();
        self.stopwatch.elapsed += elapsed;
        elapsed
    }
}
impl Drop for RunningWatch<'_> {
    fn drop(&mut self) {
        self.record_elapsed();
    }
}

#[derive(Debug, Clone)]
pub struct ElapsedStopwatch {
    watermark: Duration,
    stopwatch: Stopwatch,
}
impl ElapsedStopwatch {
    pub fn new(watermark: Duration) -> Self {
        Self {
            watermark,
            stopwatch: Stopwatch::default(),
        }
    }
    pub fn is_elapsed(&self) -> bool {
        self.watermark <= self.stopwatch.elapsed()
    }
    pub fn stopwatch(&self) -> &Stopwatch {
        &self.stopwatch
    }
    pub fn stopwatch_mut(&mut self) -> &mut Stopwatch {
        &mut self.stopwatch
    }
}

#[derive(Debug, Clone)]
pub struct Elapsed {
    watermark: Duration,
    start: Instant,
}
impl Elapsed {
    pub fn new(watermark: Duration) -> Self {
        Self {
            watermark,
            start: Instant::now(),
        }
    }
    pub fn elapsed(&self) -> Option<Duration> {
        let elapsed = self.start.elapsed();
        if self.watermark <= elapsed {
            Some(elapsed)
        } else {
            None
        }
    }
}
impl Clear for Elapsed {
    fn clear(&mut self) {
        self.start = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use crate::ops::unit::{DurationExt, HumanDuration};

    use super::*;

    #[test]
    fn test_collect_metrics() {
        let mut batch_watch = ElapsedStopwatch::new(Duration::from_secs(1));
        let mut loop_watch = ElapsedStopwatch::new(Duration::from_secs_f64(0.1));
        let mut loops = 0;
        let mut batch_running = batch_watch.stopwatch_mut().start();
        loop {
            {
                let _loop_running = loop_watch.stopwatch_mut().start();
                loops += 1;
            }
            if loop_watch.is_elapsed() {
                let latency = loop_watch.stopwatch().elapsed().div_u128(loops);
                println!("{:.1}", HumanDuration(latency));
                loop_watch.stopwatch_mut().clear();
                loops = 0;

                drop(batch_running);
                if batch_watch.is_elapsed() {
                    break;
                }
                batch_running = batch_watch.stopwatch_mut().start();
            }
        }
    }
}
