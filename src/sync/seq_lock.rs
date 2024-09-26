use core::sync::atomic::{AtomicU32, Ordering};

use super::sync_unsafe_cell::SyncUnsafeCell;

/// - single producer, multiple consumers
/// - prioritized in write
#[derive(Debug)]
pub struct SeqLock<T> {
    value: SyncUnsafeCell<T>,
    version: AtomicU32,
}
impl<T> SeqLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: SyncUnsafeCell::new(value),
            version: AtomicU32::new(0),
        }
    }

    /// # Safety
    ///
    /// Must only be accessed by one thread at a time
    pub unsafe fn store(&self, value: T) {
        let start = self.version.fetch_add(1, Ordering::Relaxed);
        let v = unsafe { self.value.get().as_mut() }.unwrap();
        *v = value;
        let end = self.version.fetch_add(1, Ordering::Release);
        assert_eq!(start & 1, 0);
        assert_eq!(end & 1, 1);
    }

    pub fn load(&self) -> Option<T>
    where
        T: Clone,
    {
        let start = self.version.load(Ordering::Acquire);
        let v = unsafe { self.value.get().as_ref() }.unwrap().clone();
        let end = self.version.load(Ordering::Relaxed);
        let start_in_write = start & 1 == 1;
        let span_thru_write = start != end;
        if start_in_write || span_thru_write {
            return None;
        }
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    const DATA_SIZE: usize = 1024;

    #[derive(Debug, Clone, Copy)]
    struct Data<T> {
        values: [T; DATA_SIZE],
    }
    impl<T> Data<T>
    where
        T: core::fmt::Debug + PartialEq + Eq + Copy,
    {
        pub fn new(value: T) -> Self {
            Self {
                values: [value; DATA_SIZE],
            }
        }
        pub fn assert(&self) {
            for v in self.values {
                assert_eq!(v, self.values[0]);
            }
        }
        pub fn get(&self) -> &[T; DATA_SIZE] {
            &self.values
        }
    }

    const N: usize = 1 << 18;
    const THREADS: usize = 1 << 3;

    #[test]
    fn test_seq_lock() {
        let l = SeqLock::new(Data::new(0));
        let l = Arc::new(l);
        let mut threads = vec![];
        for _ in 0..THREADS {
            let handle = std::thread::spawn({
                let l = l.clone();
                move || {
                    let mut n = 0;
                    loop {
                        let Some(data) = l.load() else {
                            continue;
                        };
                        n += 1;
                        data.assert();
                        if data.get()[0] + 1 == N {
                            break;
                        }
                    }
                    println!("{n}, {N}");
                    assert!(0.2 < n as f64 / N as f64);
                }
            });
            threads.push(handle);
        }
        for i in 0..N {
            unsafe { l.store(Data::new(i)) };
        }
        for handle in threads {
            handle.join().unwrap();
        }
    }
}
