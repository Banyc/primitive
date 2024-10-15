use core::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub struct Mutex1 {
    lock: AtomicBool,
}
impl Mutex1 {
    pub fn new() -> Self {
        let lock = AtomicBool::new(false);
        Self { lock }
    }
    pub fn try_lock(&self) -> bool {
        let lock = self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        if lock.is_err() {
            return false;
        }
        true
    }
    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}
impl Default for Mutex1 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod benches {
    use std::{
        sync::{Arc, Mutex},
        time::Instant,
    };

    use crate::sync::tests::RepeatedData;

    use super::*;

    const N: usize = 1 << 11;
    const THREADS: usize = 1 << 3;
    const DATA_COUNT: usize = 1 << 10;

    #[test]
    fn bench_mutex() {
        let now = Instant::now();
        let lock = Arc::new(Mutex::new(()));
        let mut threads = vec![];
        for _ in 0..THREADS {
            let handle = std::thread::spawn({
                let lock = lock.clone();
                move || {
                    for _ in 0..N {
                        let _guard = lock.lock().unwrap();
                        let data = RepeatedData::<_, DATA_COUNT>::new(0);
                        data.assert();
                    }
                }
            });
            threads.push(handle);
        }
        for handle in threads {
            handle.join().unwrap();
        }
        dbg!(now.elapsed());
    }
    #[test]
    fn bench_mutex1() {
        let now = Instant::now();
        let lock = Arc::new(Mutex1::new());
        let mut threads = vec![];
        for _ in 0..THREADS {
            let handle = std::thread::spawn({
                let lock = lock.clone();
                move || {
                    for _ in 0..N {
                        while !lock.try_lock() {
                            core::hint::spin_loop();
                        }
                        let data = RepeatedData::<_, DATA_COUNT>::new(0);
                        data.assert();
                        lock.unlock();
                    }
                }
            });
            threads.push(handle);
        }
        for handle in threads {
            handle.join().unwrap();
        }
        dbg!(now.elapsed());
    }
}
