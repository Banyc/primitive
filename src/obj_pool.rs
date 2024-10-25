use core::num::NonZeroUsize;
use std::sync::Arc;

use crate::{ops::ring::RingSpace, sync::mutex::SpinMutex, Capacity, Len};

#[derive(Debug)]
pub struct CappedStack<T> {
    buf: Vec<T>,
}
impl<T> CappedStack<T> {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }
    pub fn push(&mut self, obj: T) {
        if self.buf.len() == self.buf.capacity() {
            return;
        }
        self.buf.push(obj);
    }
    pub fn pop(&mut self) -> Option<T> {
        self.buf.pop()
    }
}
impl<T> Len for CappedStack<T> {
    fn len(&self) -> usize {
        self.buf.len()
    }
}
impl<T> Capacity for CappedStack<T> {
    fn capacity(&self) -> usize {
        self.buf.capacity()
    }
}

pub fn buf_pool<T>(capacity: usize) -> ObjectPool<Vec<T>> {
    ObjectPool::new(capacity, Vec::new, |b| b.clear())
}
#[derive(Debug)]
pub struct ObjectPool<T> {
    stack: CappedStack<T>,
    alloc: fn() -> T,
    reset: fn(&mut T),
}
impl<T> ObjectPool<T> {
    #[must_use]
    pub fn new(capacity: usize, alloc: fn() -> T, reset: fn(&mut T)) -> Self {
        Self {
            stack: CappedStack::new(capacity),
            alloc,
            reset,
        }
    }
    #[must_use]
    pub fn take(&mut self) -> T {
        self.stack.pop().unwrap_or_else(|| (self.alloc)())
    }
    pub fn put(&mut self, mut obj: T) {
        (self.reset)(&mut obj);
        self.stack.push(obj);
    }
}

pub fn arc_buf_pool<T>(capacity: usize, shards: NonZeroUsize) -> ArcObjectPool<Vec<T>> {
    ArcObjectPool::new(capacity, shards, Vec::new, |b| b.clear())
}
type ArcStacks<T> = Arc<[SpinMutex<CappedStack<T>>]>;
#[derive(Debug)]
pub struct ArcObjectPool<T> {
    stacks: ArcStacks<T>,
    next: usize,
    alloc: fn() -> T,
    reset: fn(&mut T),
}
impl<T> ArcObjectPool<T> {
    #[must_use]
    pub fn new(capacity: usize, shards: NonZeroUsize, alloc: fn() -> T, reset: fn(&mut T)) -> Self {
        let mut stacks = vec![];
        for _ in 0..shards.get() {
            stacks.push(SpinMutex::new(CappedStack::new(capacity)));
        }
        Self {
            stacks: stacks.into(),
            next: 0,
            alloc,
            reset,
        }
    }
    #[must_use]
    pub fn take(&mut self) -> T {
        let shard = self.next;
        if 1 < self.stacks.len() {
            self.next = self.next.ring_add(1, self.stacks.len() - 1);
        }
        self.stacks[shard]
            .lock()
            .pop()
            .unwrap_or_else(|| (self.alloc)())
    }
    #[must_use]
    pub fn recycler(&self) -> ObjectRecycler<T> {
        ObjectRecycler {
            stacks: Arc::clone(&self.stacks),
            next: self.next,
            reset: self.reset,
        }
    }
}
#[derive(Debug)]
pub struct ObjectRecycler<T> {
    stacks: ArcStacks<T>,
    next: usize,
    reset: fn(&mut T),
}
impl<T> ObjectRecycler<T> {
    pub fn put(&mut self, mut obj: T) {
        let shard = self.next;
        if 1 < self.stacks.len() {
            self.next = self.next.ring_add(1, self.stacks.len() - 1);
        }
        (self.reset)(&mut obj);
        self.stacks[shard].lock().push(obj);
    }
}
impl<T> Clone for ObjectRecycler<T> {
    fn clone(&self) -> Self {
        Self {
            stacks: Arc::clone(&self.stacks),
            next: self.next,
            reset: self.reset,
        }
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use super::*;

    const N: usize = 2 << 18;
    const DATA_SIZE: usize = 2;

    #[derive(Default)]
    struct Data {
        _buf: [u8; DATA_SIZE],
    }

    #[bench]
    fn bench_arc_pool(bencher: &mut test::Bencher) {
        let mut in_use = vec![];
        let mut pool = arc_buf_pool(u32::MAX as usize, NonZeroUsize::new(1).unwrap());
        let mut recycler = pool.recycler();
        bencher.iter(|| {
            for _ in 0..N {
                let mut buf = pool.take();
                buf.push(Data::default());
                in_use.push(buf);
            }
            for _ in 0..N {
                let buf = in_use.pop().unwrap();
                recycler.put(buf);
            }
        });
    }

    #[bench]
    fn bench_pool(bencher: &mut test::Bencher) {
        let mut in_use = vec![];
        let mut pool = buf_pool(u32::MAX as usize);
        bencher.iter(|| {
            for _ in 0..N {
                let mut buf = pool.take();
                buf.push(Data::default());
                in_use.push(buf);
            }
            for _ in 0..N {
                let buf = in_use.pop().unwrap();
                pool.put(buf);
            }
        });
    }

    #[bench]
    fn bench_alloc(bencher: &mut test::Bencher) {
        let mut in_use = vec![];
        bencher.iter(|| {
            for _ in 0..N {
                let buf = vec![Data::default()];
                in_use.push(buf);
            }
            for _ in 0..N {
                in_use.pop().unwrap();
            }
        });
    }
}
