use core::{
    mem::MaybeUninit,
    sync::atomic::{fence, AtomicUsize, Ordering},
};
use std::{ops::Deref, sync::Arc};

use crate::wrap::RingSpace;

use super::seq_lock::SeqLock;

/// - message overriding
#[derive(Debug)]
pub struct SpmcQueue<T, const N: usize> {
    ring: [SeqLock<MaybeUninit<T>>; N],
    next: AtomicUsize,
}
impl<T, const N: usize> SpmcQueue<T, N> {
    pub fn new() -> Self {
        assert!(1 < N);
        let ring = (0..N)
            .map(|_| SeqLock::new(MaybeUninit::uninit()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let next = AtomicUsize::new(0);
        Self { ring, next }
    }

    /// # Safety
    ///
    /// Must only be accessed by one thread at a time
    pub unsafe fn push(&self, value: T)
    where
        T: Copy,
    {
        let next = self.next.load(Ordering::Acquire);
        let value = MaybeUninit::new(value);
        let lock = &self.ring[next];
        unsafe { lock.store(value) };
        let next = next.ring_add(1, N - 1);
        self.next.store(next, Ordering::Release);
    }

    pub fn next_version(&self) -> (usize, u32) {
        let next = self.next.load(Ordering::Acquire);
        let version = self.ring[next].version();
        fence(Ordering::Release);
        let new_next = self.next.load(Ordering::Relaxed);
        if version & 1 == 1 {
            let min_ver = version.wrapping_add(1);
            return (next, min_ver);
        }
        let min_ver = if next == new_next {
            version.wrapping_add(2)
        } else {
            version
        };
        (next, min_ver)
    }
    pub fn load(&self, position: usize, min_ver: u32) -> Option<(T, u32)>
    where
        T: Copy,
    {
        let lock = &self.ring[position];
        let (value, ver) = lock.load()?;
        let ahead_of_write = ver.wrapping_add(2) == min_ver;
        if ahead_of_write {
            return None;
        }
        let value = unsafe { value.assume_init() };
        Some((value, ver))
    }
}
impl<T, const N: usize> Default for SpmcQueue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn safe_smpc_queue<T, const N: usize>() -> (
    SpmcQueueReader<T, N, Arc<SpmcQueue<T, N>>>,
    SpmcQueueWriter<T, N>,
) {
    let queue = SpmcQueue::new();
    let queue = Arc::new(queue);
    let reader = SpmcQueueReader::new(Arc::clone(&queue));
    let writer = SpmcQueueWriter { queue };
    (reader, writer)
}
#[derive(Debug)]
pub struct SpmcQueueWriter<T, const N: usize> {
    queue: Arc<SpmcQueue<T, N>>,
}
impl<T, const N: usize> SpmcQueueWriter<T, N>
where
    T: Copy,
{
    pub fn push(&mut self, value: T) {
        unsafe { self.queue.push(value) };
    }
}
#[derive(Debug, Clone)]
pub struct SpmcQueueReader<T, const N: usize, Q>
where
    Q: Deref<Target = SpmcQueue<T, N>>,
{
    queue: Q,
    position: usize,
    min_ver: u32,
}
impl<T, const N: usize, Q> SpmcQueueReader<T, N, Q>
where
    Q: Deref<Target = SpmcQueue<T, N>>,
{
    pub fn new(queue: Q) -> Self {
        let (position, min_ver) = queue.next_version();
        Self {
            queue,
            position,
            min_ver,
        }
    }
    pub fn pop(&mut self) -> Option<T>
    where
        T: Copy,
    {
        let (val, ver) = self.queue.load(self.position, self.min_ver)?;
        let ver_bump = self.min_ver != ver;
        let at_ver_start_pos = 0 == self.position;
        if !ver_bump && at_ver_start_pos {
            return None;
        }
        self.min_ver = ver;
        self.position = self.position.ring_add(1, N - 1);
        Some(val)
    }
}

#[cfg(test)]
mod tests {
    use crate::sync::tests::RepeatedData;

    use super::*;

    const DATA_COUNT: usize = 1024;
    const N: usize = 1 << 18;
    const THREADS: usize = 1 << 3;
    // const RATE: f64 = 0.2;
    const QUEUE_SIZE: usize = 2;

    #[test]
    fn test_smpc_queue() {
        let (rdr, mut wtr) = safe_smpc_queue::<RepeatedData<_, DATA_COUNT>, QUEUE_SIZE>();
        let mut threads = vec![];
        for _ in 0..THREADS {
            let handle = std::thread::spawn({
                let mut rdr = rdr.clone();
                move || {
                    let mut n = 0;
                    let mut prev: Option<usize> = None;
                    loop {
                        let Some(data) = rdr.pop() else {
                            continue;
                        };
                        n += 1;
                        data.assert();
                        let value = data.get()[0];
                        if let Some(prev) = prev {
                            assert!(prev < value, "{prev}; {value}; {rdr:?}");
                        }
                        prev = Some(value);
                        if value + 1 == N {
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
            let data = RepeatedData::new(i);
            wtr.push(data);
        }
        for handle in threads {
            handle.join().unwrap();
        }
    }
}
