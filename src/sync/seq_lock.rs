use core::sync::atomic::{AtomicU32, Ordering, fence};
use std::sync::Arc;

use atomic_memcpy::{atomic_load, atomic_store};
use bytemuck::NoUninit;

use super::sync_unsafe_cell::SyncUnsafeCell;

/// - single producer, multiple consumers
/// - prioritized in write
#[derive(Debug)]
#[repr(C)]
pub struct SeqLock<T: NoUninit> {
    value: SyncUnsafeCell<T>,
    version: AtomicU32,
}
impl<T: NoUninit> SeqLock<T> {
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value: SyncUnsafeCell::new(value),
            version: AtomicU32::new(0),
        }
    }

    /// # Safety
    ///
    /// - Must only be accessed by one thread at a time.
    pub unsafe fn store(&self, value: T) {
        let prev_start = self.version.fetch_add(1, Ordering::Acquire);
        unsafe { atomic_store(self.value.get(), value, Ordering::Release) };
        let prev_end = self.version.fetch_add(1, Ordering::Release);
        assert_eq!(prev_start & 1, 0);
        assert_eq!(prev_end & 1, 1);
    }

    /// Return [`None`] if the value is being modified or been modified during read
    #[must_use]
    pub fn load(&self) -> Option<(T, u32)> {
        let start = self.version.load(Ordering::Acquire);
        let v = unsafe { atomic_load(self.value.get(), Ordering::Acquire) };
        fence(Ordering::Release);
        let end = self.version.load(Ordering::Acquire);
        let start_in_write = start & 1 == 1;
        let span_thru_write = start != end;
        if start_in_write || span_thru_write {
            return None;
        }
        Some((unsafe { v.assume_init() }, start))
    }

    #[must_use]
    pub fn version(&self) -> u32 {
        self.version.load(Ordering::SeqCst)
    }
}

pub fn safe_seq_lock<T: NoUninit>(value: T) -> (SeqLockReader<T>, SeqLockWriter<T>) {
    let lock = SeqLock::new(value);
    let lock = Arc::new(lock);
    let reader = SeqLockReader {
        lock: Arc::clone(&lock),
    };
    let writer = SeqLockWriter {
        lock: Arc::clone(&lock),
    };
    (reader, writer)
}
#[derive(Debug, Clone)]
pub struct SeqLockReader<T: NoUninit> {
    lock: Arc<SeqLock<T>>,
}
impl<T: NoUninit> SeqLockReader<T> {
    pub fn load(&self) -> Option<T> {
        self.lock.load().map(|(x, _)| x)
    }
}
#[derive(Debug)]
pub struct SeqLockWriter<T: NoUninit> {
    lock: Arc<SeqLock<T>>,
}
impl<T: NoUninit> SeqLockWriter<T> {
    pub fn store(&mut self, value: T) {
        unsafe { self.lock.store(value) };
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::sync::tests::RepeatedData;

    use super::*;

    const DATA_COUNT: usize = 1024;
    #[cfg(miri)]
    const N: usize = 1 << 2;
    #[cfg(not(miri))]
    const N: usize = 1 << 18;
    const THREADS: usize = 1 << 3;
    // const RATE: f64 = 0.3;

    #[test]
    fn test_seq_lock() {
        let l = SeqLock::new(RepeatedData::<_, DATA_COUNT>::new(0));
        let l = Arc::new(l);
        let mut threads = vec![];
        for _ in 0..THREADS {
            let handle = std::thread::spawn({
                let l = l.clone();
                move || {
                    let mut n = 0;
                    loop {
                        let Some((data, _)) = l.load() else {
                            continue;
                        };
                        n += 1;
                        data.assert();
                        if data.get()[0] + 1 == N {
                            break;
                        }
                    }
                    let rate = n as f64 / N as f64;
                    println!("{n}; {N}; {rate}");
                    // assert!(RATE < rate);
                }
            });
            threads.push(handle);
        }
        for i in 0..N {
            unsafe { l.store(RepeatedData::new(i)) };
        }
        for handle in threads {
            handle.join().unwrap();
        }
    }
}
