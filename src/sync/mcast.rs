use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{fence, AtomicUsize, Ordering},
};
use std::sync::Arc;

use crate::ops::{dyn_ref::DynRef, ring::RingSpace};

use super::{mutex::Mutex1, seq_lock::SeqLock};

/// - message overwriting
#[derive(Debug)]
pub struct SpMcast<T, const N: usize> {
    ring: [SeqLock<MaybeUninit<T>>; N],
    next: AtomicUsize,
}
impl<T, const N: usize> SpMcast<T, N> {
    pub const fn new() -> Self {
        const {
            assert!(1 < N);
        }
        let ring = [const { SeqLock::new(MaybeUninit::uninit()) }; N];
        let next = AtomicUsize::new(0);
        Self { ring, next }
    }

    pub fn next_version(&self) -> (usize, MinVer) {
        let next = self.next.load(Ordering::Acquire);
        let version = self.ring[next].version();
        fence(Ordering::Release);
        let new_next = self.next.load(Ordering::Relaxed);
        if version & 1 == 1 {
            let min_ver = version.wrapping_add(1);
            return (next, MinVer(min_ver));
        }
        let min_ver = if next == new_next {
            version.wrapping_add(2)
        } else {
            version
        };
        (next, MinVer(min_ver))
    }
}
impl<T, const N: usize> SpMcast<T, N>
where
    T: Copy,
{
    /// # Safety
    ///
    /// Must only be accessed by one thread at a time
    pub unsafe fn push(&self, value: T) {
        let next = self.next.load(Ordering::Acquire);
        let value = MaybeUninit::new(value);
        let lock = &self.ring[next];
        unsafe { lock.store(value) };
        let next = next.ring_add(1, N - 1);
        self.next.store(next, Ordering::Release);
    }

    /// # Safety
    ///
    /// `min_ver` must be received from [`Self::next_version()`] and later updated by [`Self::load()`] both from this instance
    pub unsafe fn load(&self, position: usize, min_ver: MinVer) -> Option<(T, MinVer)> {
        let lock = &self.ring[position];
        let (value, ver) = lock.load()?;
        let ahead_of_write = ver.wrapping_add(2) == min_ver.0;
        if ahead_of_write {
            return None;
        }
        let value = unsafe { value.assume_init() };
        let min_ver = MinVer(ver);
        Some((value, min_ver))
    }
}
impl<T, const N: usize> Default for SpMcast<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinVer(u32);

pub fn spmcast_channel<T, const N: usize>(
) -> (SpMcastReader<T, N, Arc<SpMcast<T, N>>>, SpMcastWriter<T, N>) {
    let queue = SpMcast::new();
    let queue = Arc::new(queue);
    let queue_ref = DynRef::new(Arc::clone(&queue), |q| q.as_ref());
    let reader = SpMcastReader::new(queue_ref);
    let writer = SpMcastWriter { queue };
    (reader, writer)
}
#[derive(Debug)]
pub struct SpMcastWriter<T, const N: usize> {
    queue: Arc<SpMcast<T, N>>,
}
impl<T, const N: usize> SpMcastWriter<T, N>
where
    T: Copy,
{
    pub fn push(&mut self, value: T) {
        unsafe { self.queue.push(value) };
    }
}
#[derive(Debug, Clone)]
pub struct SpMcastReader<T, const N: usize, Q> {
    queue: DynRef<Q, SpMcast<T, N>>,
    position: usize,
    min_ver: MinVer,
    read_once: bool,
    _item: PhantomData<T>,
}
impl<T, const N: usize, Q> SpMcastReader<T, N, Q> {
    pub fn new(queue: DynRef<Q, SpMcast<T, N>>) -> Self {
        let (position, min_ver) = queue.convert().next_version();
        Self {
            queue,
            position,
            min_ver,
            read_once: false,
            _item: PhantomData,
        }
    }
    pub fn pop(&mut self) -> Option<T>
    where
        T: Copy,
    {
        let (val, ver) = unsafe { self.queue.convert().load(self.position, self.min_ver) }?;
        let ver_bump = self.min_ver != ver;
        let at_ver_start_pos = 0 == self.position;
        if !ver_bump && at_ver_start_pos && self.read_once {
            return None;
        }
        self.min_ver = ver;
        self.position = self.position.ring_add(1, N - 1);
        self.read_once = true;
        Some(val)
    }
}

/// - message overriding
#[derive(Debug)]
pub struct MpMcast<T, const N: usize> {
    write: Mutex1,
    queue: SpMcast<T, N>,
}
impl<T, const N: usize> MpMcast<T, N> {
    pub const fn new() -> Self {
        let write = Mutex1::new();
        let queue = SpMcast::new();
        Self { write, queue }
    }
    pub const fn queue(&self) -> &SpMcast<T, N> {
        &self.queue
    }
}
impl<T, const N: usize> Default for MpMcast<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> MpMcast<T, N>
where
    T: Copy,
{
    pub fn try_push(&self, value: T) -> bool {
        if !self.write.try_lock() {
            return false;
        }
        unsafe { self.queue.push(value) };
        self.write.unlock();
        true
    }
}
#[allow(clippy::type_complexity)]
pub fn mpmcast_channel<T, const N: usize>(
) -> (MpMcastReader<T, N, Arc<MpMcast<T, N>>>, Arc<MpMcast<T, N>>) {
    let queue = MpMcast::new();
    let queue = Arc::new(queue);
    let reader = MpMcastReader::new(DynRef::new(queue.clone(), |q| q.as_ref()));
    let writer = queue;
    (reader, writer)
}
#[derive(Debug, Clone)]
pub struct MpMcastReader<T, const N: usize, Q> {
    reader: SpMcastReader<T, N, DynRef<Q, MpMcast<T, N>>>,
}
impl<T, const N: usize, Q> MpMcastReader<T, N, Q> {
    pub fn new(queue: DynRef<Q, MpMcast<T, N>>) -> Self {
        let queue_ref = DynRef::new(queue, |q| q.convert().queue());
        let reader = SpMcastReader::new(queue_ref);
        Self { reader }
    }
}
impl<T, const N: usize, Q> MpMcastReader<T, N, Q>
where
    T: Copy,
{
    pub fn pop(&mut self) -> Option<T> {
        self.reader.pop()
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
    fn test_spmcast_queue() {
        let (rdr, mut wtr) = spmcast_channel::<RepeatedData<_, DATA_COUNT>, QUEUE_SIZE>();
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

    #[test]
    fn test_transmute() {
        type Queue = MpMcast<RepeatedData<usize, DATA_COUNT>, QUEUE_SIZE>;
        const BUF_SIZE: usize = core::mem::size_of::<Queue>();
        type Buf = [u8; BUF_SIZE];
        let mut buf = Box::new([0; BUF_SIZE]);
        let buf = {
            let queue: Queue = MpMcast::new();
            let bytes = unsafe { core::mem::transmute::<Queue, Buf>(queue) };
            buf.copy_from_slice(&bytes);
            buf.into()
        };
        let queue = unsafe { core::mem::transmute::<Arc<Buf>, Arc<Queue>>(buf) };
        test_mpmcast_queue(queue);
    }
    fn test_mpmcast_queue<const QUEUE_SIZE: usize>(
        queue: Arc<MpMcast<RepeatedData<usize, DATA_COUNT>, QUEUE_SIZE>>,
    ) {
        let rdr = MpMcastReader::new(DynRef::new(queue.clone(), |q| q.as_ref()));
        let wtr = queue;
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
            while !wtr.try_push(data) {}
        }
        for handle in threads {
            handle.join().unwrap();
        }
    }
}
