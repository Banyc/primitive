use std::time::{Duration, Instant};

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

#[derive(Debug)]
pub struct RunningWatch<'a> {
    stopwatch: &'a mut Stopwatch,
    start: Instant,
}
impl Drop for RunningWatch<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.stopwatch.elapsed += elapsed;
    }
}
