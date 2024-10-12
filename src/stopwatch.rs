use std::time::{Duration, Instant};

use crate::Clear;

#[derive(Debug, Clone)]
pub struct Stopwatch {
    elapsed: Duration,
    start: Option<Instant>,
}
impl Stopwatch {
    pub fn new(elapsed: Duration) -> Self {
        Self {
            elapsed,
            start: None,
        }
    }
    pub fn start_scoped(&mut self) -> RunningWatch<'_> {
        let now = Instant::now();
        if let Some(start) = self.start.take() {
            self.elapsed += now - start;
        }
        RunningWatch {
            stopwatch: self,
            start: now,
        }
    }
    pub fn start(&mut self) {
        if self.start.is_some() {
            return;
        }
        self.start = Some(Instant::now());
    }
    pub fn pause(&mut self) {
        let Some(start) = self.start.take() else {
            return;
        };
        self.elapsed += start.elapsed();
    }
    pub fn elapsed(&self) -> Duration {
        self.elapsed + self.start.map(|start| start.elapsed()).unwrap_or_default()
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
    use std::{num::NonZeroUsize, sync::mpsc};

    use crate::{
        bench::ExpMovVar,
        ops::unit::{DurationExt, HumanDuration},
        sync::spmc::{self, spmc_channel},
    };

    use super::*;

    #[test]
    fn test_collect_metrics() {
        let mut batch_watch = ElapsedStopwatch::new(Duration::from_secs(1));
        let mut loop_watch = ElapsedStopwatch::new(Duration::from_secs_f64(0.1));
        let mut loops = 0;
        let mut batch_running = batch_watch.stopwatch_mut().start_scoped();
        loop {
            {
                let _loop_running = loop_watch.stopwatch_mut().start_scoped();
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
                batch_running = batch_watch.stopwatch_mut().start_scoped();
            }
        }
    }

    pub trait ChanSend<T> {
        fn send(&mut self, msg: T) -> Result<(), T>;
    }
    impl<T> ChanSend<T> for mpsc::SyncSender<T> {
        fn send(&mut self, msg: T) -> Result<(), T> {
            mpsc::SyncSender::send(self, msg).map_err(|e| e.0)
        }
    }
    impl<T: Copy, const N: usize> ChanSend<T> for spmc::SpmcQueueWriter<T, N> {
        fn send(&mut self, msg: T) -> Result<(), T> {
            spmc::SpmcQueueWriter::push(self, msg);
            Ok(())
        }
    }

    #[allow(unused)]
    pub trait ChanRecv<T> {
        fn try_recv(&mut self) -> Result<T, TryRecvError>;
        fn recv(&mut self) -> Result<T, ()>;
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TryRecvError {
        Disconnected,
        Empty,
    }
    impl<T> ChanRecv<T> for mpsc::Receiver<T> {
        fn recv(&mut self) -> Result<T, ()> {
            mpsc::Receiver::recv(self).map_err(|_| ())
        }
        fn try_recv(&mut self) -> Result<T, TryRecvError> {
            mpsc::Receiver::try_recv(self).map_err(|e| match e {
                mpsc::TryRecvError::Empty => TryRecvError::Empty,
                mpsc::TryRecvError::Disconnected => TryRecvError::Disconnected,
            })
        }
    }
    impl<T: Copy, const N: usize, Q> ChanRecv<T> for spmc::SpmcQueueReader<T, N, Q> {
        fn recv(&mut self) -> Result<T, ()> {
            loop {
                let Some(msg) = spmc::SpmcQueueReader::pop(self) else {
                    continue;
                };
                return Ok(msg);
            }
        }
        fn try_recv(&mut self) -> Result<T, TryRecvError> {
            spmc::SpmcQueueReader::pop(self).ok_or(TryRecvError::Empty)
        }
    }

    fn bench_channel_latency(
        mut tx: impl ChanSend<Instant> + Send + 'static,
        mut rx: impl ChanRecv<Instant> + Send + 'static,
    ) {
        let mut elapsed = Elapsed::new(Duration::from_millis(200));
        std::thread::spawn(move || loop {
            tx.send(Instant::now()).unwrap();
        });
        std::thread::spawn(move || {
            let mut emvar = ExpMovVar::from_periods(NonZeroUsize::new(16).unwrap());
            let mut ema_watch = Stopwatch::default();
            let mut ema_count = 0;
            while let Ok(time) = rx.recv() {
                ema_watch.start();
                let latency = time.elapsed();
                emvar.update(latency.as_secs_f64());
                ema_count += 1;
                ema_watch.pause();
                if elapsed.elapsed().is_some() && emvar.var().get().is_some() {
                    elapsed.clear();
                    println!(
                        "mean: {:.1}; var: {:.1}; stats overhead: {:.1}",
                        HumanDuration(Duration::from_secs_f64(emvar.mean().get().unwrap())),
                        HumanDuration(Duration::from_secs_f64(emvar.var().get().unwrap())),
                        HumanDuration(ema_watch.elapsed().div_u128(ema_count))
                    );
                    ema_watch.clear();
                    ema_count = 0;
                }
            }
        });
        std::thread::sleep(Duration::from_secs(10));
    }

    #[test]
    #[ignore]
    fn bench_latency_std_mpsc() {
        let (tx, rx) = mpsc::sync_channel(0);
        bench_channel_latency(tx, rx);
    }
    #[test]
    #[ignore]
    fn bench_latency_spmc() {
        let (rx, tx) = spmc_channel::<Instant, 2>();
        bench_channel_latency(tx, rx);
    }
}
