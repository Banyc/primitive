use core::{
    mem::MaybeUninit,
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::{ops::ring::RingSpace, sync::mutex::SpinMutex};

use super::stack::{DynStack, Stack};

pub fn buf_pool<T>(capacity: Option<usize>) -> ObjectPool<Vec<T>> {
    ObjectPool::new(capacity, Vec::new, |b| b.clear())
}
#[derive(Debug)]
pub struct ObjectPool<T> {
    stack: DynStack<T>,
    alloc: fn() -> T,
    reset: fn(&mut T),
}
impl<T> ObjectPool<T> {
    #[must_use]
    pub fn new(capacity: Option<usize>, alloc: fn() -> T, reset: fn(&mut T)) -> Self {
        Self {
            stack: DynStack::new(capacity),
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

pub fn arc_buf_pool<T>(capacity: Option<usize>, shards: NonZeroUsize) -> ArcObjectPool<Vec<T>> {
    ArcObjectPool::new(capacity, shards, Vec::new, |b| b.clear())
}
type ArcStacks<T> = Arc<[SpinMutex<DynStack<T>>]>;
#[derive(Debug)]
pub struct ArcObjectPool<T> {
    stacks: ArcStacks<T>,
    next: AtomicUsize,
    alloc: fn() -> T,
    reset: fn(&mut T),
}
impl<T> ArcObjectPool<T> {
    #[must_use]
    pub fn new(
        capacity: Option<usize>,
        shards: NonZeroUsize,
        alloc: fn() -> T,
        reset: fn(&mut T),
    ) -> Self {
        let mut stacks = vec![];
        for _ in 0..shards.get() {
            stacks.push(SpinMutex::new(DynStack::new(capacity)));
        }
        Self {
            stacks: stacks.into(),
            next: AtomicUsize::new(0),
            alloc,
            reset,
        }
    }
    #[must_use]
    pub fn take(&self) -> T {
        self.stacks[self.shard_incr()]
            .lock()
            .pop()
            .unwrap_or_else(|| (self.alloc)())
    }
    #[must_use]
    pub fn take_scoped(&self) -> ObjectScoped<T> {
        ObjectScoped::new(self.recycler(), self.take())
    }
    pub fn put(&self, mut obj: T) {
        (self.reset)(&mut obj);
        self.stacks[self.shard_incr()].lock().push(obj);
    }
    #[must_use]
    pub fn recycler(&self) -> ObjectRecycler<T> {
        ObjectRecycler {
            stacks: Arc::clone(&self.stacks),
            next: self.shard(),
            reset: self.reset,
        }
    }
    #[must_use]
    fn shard(&self) -> usize {
        match self.stacks.len() {
            1 => 0,
            _ => self.next.load(Ordering::Relaxed),
        }
    }
    #[must_use]
    fn shard_incr(&self) -> usize {
        match self.stacks.len() {
            1 => 0,
            _ => {
                let shard = self.next.load(Ordering::Relaxed);
                let next = shard.ring_add(1, self.stacks.len() - 1);
                self.next.store(next, Ordering::Relaxed);
                shard
            }
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

#[derive(Debug)]
pub struct ObjectScoped<T> {
    recycler: ObjectRecycler<T>,
    obj: MaybeUninit<T>,
}
impl<T> ObjectScoped<T> {
    pub fn new(recycler: ObjectRecycler<T>, obj: T) -> Self {
        Self {
            recycler,
            obj: MaybeUninit::new(obj),
        }
    }
}
impl<T> Deref for ObjectScoped<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.obj.assume_init_ref() }
    }
}
impl<T> DerefMut for ObjectScoped<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.obj.assume_init_mut() }
    }
}
impl<T> Drop for ObjectScoped<T> {
    fn drop(&mut self) {
        let obj = core::mem::replace(&mut self.obj, MaybeUninit::uninit());
        let obj = unsafe { obj.assume_init() };
        self.recycler.put(obj);
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
    #[ignore]
    fn bench_lockfree_object_pool_owned(bencher: &mut test::Bencher) {
        let mut in_use = vec![];
        let pool = Arc::new(lockfree_object_pool::LinearObjectPool::new(
            Vec::new,
            |buf| buf.clear(),
        ));
        bencher.iter(|| {
            for _ in 0..N {
                let mut buf = pool.pull_owned();
                buf.push(Data::default());
                in_use.push(buf);
            }
            for _ in 0..N {
                in_use.pop().unwrap();
            }
        });
    }

    #[bench]
    fn bench_arc_pool_scoped(bencher: &mut test::Bencher) {
        let mut in_use = vec![];
        let pool = arc_buf_pool(None, NonZeroUsize::new(4).unwrap());
        bencher.iter(|| {
            for _ in 0..N {
                let mut buf = pool.take_scoped();
                buf.push(Data::default());
                in_use.push(buf);
            }
            for _ in 0..N {
                in_use.pop().unwrap();
            }
        });
    }

    #[bench]
    fn bench_arc_pool(bencher: &mut test::Bencher) {
        let mut in_use = vec![];
        let pool = arc_buf_pool(None, NonZeroUsize::new(1).unwrap());
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
        let mut pool = buf_pool(None);
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
