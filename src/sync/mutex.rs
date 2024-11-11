use core::{
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
    sync::atomic::Ordering,
};

use super::sync_unsafe_cell::SyncUnsafeCell;

#[derive(Debug)]
pub struct Mutex1 {
    lock: AtomicBool,
}
impl Mutex1 {
    pub const fn new() -> Self {
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

#[derive(Debug)]
pub struct SpinMutex<T> {
    lock: Mutex1,
    value: SyncUnsafeCell<T>,
}
impl<T> SpinMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            lock: Mutex1::new(),
            value: SyncUnsafeCell::new(value),
        }
    }
    pub fn lock(&self) -> SpinMutexScoped<T> {
        while !self.lock.try_lock() {
            core::hint::spin_loop();
        }
        SpinMutexScoped { mutex: self }
    }
    pub fn try_lock(&self) -> Option<SpinMutexScoped<T>> {
        if !self.lock.try_lock() {
            return None;
        }
        Some(SpinMutexScoped { mutex: self })
    }
}
#[derive(Debug)]
pub struct SpinMutexScoped<'a, T> {
    mutex: &'a SpinMutex<T>,
}
impl<T> Deref for SpinMutexScoped<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}
impl<T> DerefMut for SpinMutexScoped<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.value.get() }
    }
}
impl<T> Drop for SpinMutexScoped<'_, T> {
    fn drop(&mut self) {
        self.mutex.lock.unlock();
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
