use core::time::Duration;
use std::time::Instant;

use crate::ops::clear::Clear;

#[derive(Debug, Clone)]
pub struct Stopwatch {
    elapsed: Duration,
    start: Option<Instant>,
}
impl Stopwatch {
    pub const fn new(elapsed: Duration) -> Self {
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
    pub const fn start(&self) -> Instant {
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
    pub const fn stopwatch(&self) -> &Stopwatch {
        &self.stopwatch
    }
    pub const fn stopwatch_mut(&mut self) -> &mut Stopwatch {
        &mut self.stopwatch
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroUsize;
    use std::sync::mpsc;

    use bytemuck::NoUninit;

    use crate::{
        analysis::bench::ExpMovVar,
        ops::unit::{DurationExt, HumanDuration},
        sync::mcast,
        time::timer::Timer,
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
    impl<T> ChanSend<T> for crossbeam::channel::Sender<T> {
        fn send(&mut self, msg: T) -> Result<(), T> {
            crossbeam::channel::Sender::send(self, msg).map_err(|e| e.0)
        }
    }
    impl<T: NoUninit, const N: usize> ChanSend<T> for mcast::SpMcastWriter<T, N> {
        fn send(&mut self, msg: T) -> Result<(), T> {
            mcast::SpMcastWriter::push(self, msg);
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
    impl<T> ChanRecv<T> for crossbeam::channel::Receiver<T> {
        fn recv(&mut self) -> Result<T, ()> {
            crossbeam::channel::Receiver::recv(self).map_err(|_| ())
        }
        fn try_recv(&mut self) -> Result<T, TryRecvError> {
            crossbeam::channel::Receiver::try_recv(self).map_err(|e| match e {
                crossbeam::channel::TryRecvError::Empty => TryRecvError::Empty,
                crossbeam::channel::TryRecvError::Disconnected => TryRecvError::Disconnected,
            })
        }
    }
    impl<T: NoUninit, const N: usize, Q> ChanRecv<T> for mcast::SpMcastReader<T, N, Q> {
        fn recv(&mut self) -> Result<T, ()> {
            loop {
                let Some(msg) = mcast::SpMcastReader::pop(self) else {
                    continue;
                };
                return Ok(msg);
            }
        }
        fn try_recv(&mut self) -> Result<T, TryRecvError> {
            mcast::SpMcastReader::pop(self).ok_or(TryRecvError::Empty)
        }
    }

    fn bench_channel_latency(
        mut tx: impl ChanSend<Instant> + Send + 'static,
        mut rx: impl ChanRecv<Instant> + Send + 'static,
    ) {
        std::thread::spawn(move || {
            loop {
                tx.send(Instant::now()).unwrap();
            }
        });
        std::thread::spawn(move || {
            let mut report = LatencyReport::default();
            while let Ok(time) = rx.recv() {
                report.update(time);
            }
        });
        std::thread::sleep(Duration::from_secs(10));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn bench_channel_latency_tokio() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        tokio::task::spawn(async move {
            loop {
                tx.send(Instant::now()).await.unwrap();
            }
        });
        tokio::task::spawn(async move {
            let mut report = LatencyReport::default();
            while let Some(time) = rx.recv().await {
                report.update(time);
            }
        });
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
    #[test]
    #[ignore]
    fn bench_channel_latency_std_mpsc() {
        let (tx, rx) = mpsc::sync_channel(0);
        bench_channel_latency(tx, rx);
    }
    #[test]
    #[ignore]
    fn bench_channel_latency_crossbeam() {
        let (tx, rx) = crossbeam::channel::bounded::<Instant>(0);
        bench_channel_latency(tx, rx);
    }
    // #[test]
    // #[ignore]
    // fn bench_channel_latency_spmcast() {
    //     let (rx, tx) = spmcast_channel::<Instant, 2>(|_| Instant::now());
    //     bench_channel_latency(tx, rx);
    // }

    pub struct LatencyReport {
        emvar: ExpMovVar<f64>,
        ema_watch: Stopwatch,
        ema_count: u128,
        every: Duration,
        timer: Timer,
    }
    impl LatencyReport {
        pub fn update(&mut self, start: Instant) {
            self.ema_watch.start();
            let latency = start.elapsed();
            self.emvar.update(latency.as_secs_f64());
            self.ema_count += 1;
            self.ema_watch.pause();
            let now = Instant::now();
            let (set_off, _) = self.timer.ensure_started_and_check(self.every, now);
            if set_off && self.emvar.var().get().is_some() {
                self.timer.restart(now);
                println!(
                    "mean: {:.1}; var: {:.1}; stats overhead: {:.1}",
                    HumanDuration(Duration::from_secs_f64(self.emvar.mean().get().unwrap())),
                    HumanDuration(Duration::from_secs_f64(self.emvar.var().get().unwrap())),
                    HumanDuration(self.ema_watch.elapsed().div_u128(self.ema_count))
                );
                self.ema_watch.clear();
                self.ema_count = 0;
            }
        }
    }
    impl Default for LatencyReport {
        fn default() -> Self {
            Self {
                emvar: ExpMovVar::from_periods(NonZeroUsize::new(16 * 1024).unwrap()),
                ema_watch: Default::default(),
                ema_count: Default::default(),
                every: Duration::from_millis(200),
                timer: Timer::new(),
            }
        }
    }
}
