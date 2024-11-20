use core::{marker::PhantomData, mem::MaybeUninit, num::NonZeroUsize};

use crate::{
    ops::{
        clear::Clear,
        len::{Capacity, Len, LenExt},
        list::ListMut,
        ring::RingSpace,
        slice::{AsSlice, AsSliceMut},
    },
    set::bit_set::BitSet,
};

#[derive(Debug, Clone, Copy)]
pub struct CapQueuePointer {
    #[cfg(debug_assertions)]
    cap: usize,
    prev_head: usize,
    next_tail: usize,
}
impl CapQueuePointer {
    #[must_use]
    pub const fn new(#[cfg(debug_assertions)] cap: usize) -> Self {
        Self {
            #[cfg(debug_assertions)]
            cap,
            prev_head: 0,
            next_tail: 1,
        }
    }
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn cap(&self) -> usize {
        self.cap
    }
    #[must_use]
    pub fn head(&self, cap: usize) -> usize {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        self.prev_head.ring_add(1, cap)
    }
    #[must_use]
    pub fn len(&self, cap: usize) -> usize {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        let dist = self.next_tail.ring_sub(self.prev_head, cap);
        dist.checked_sub(1).unwrap_or(cap)
    }
    #[must_use]
    fn is_empty(&self, cap: usize) -> bool {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        self.len(cap) == 0
    }
    #[must_use]
    pub fn enqueue(&mut self, cap: usize) -> usize {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        if self.prev_head == self.next_tail {
            panic!("out of buffer space");
        }
        let index = self.next_tail;
        self.next_tail = self.next_tail.ring_add(1, cap);
        index
    }
    #[must_use]
    pub fn batch_enqueue(
        &mut self,
        amount: NonZeroUsize,
        cap: usize,
    ) -> (core::ops::Range<usize>, Option<core::ops::Range<usize>>) {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        let space = cap - self.len(cap);
        assert!(amount.get() <= space);
        let start = self.next_tail;
        self.next_tail = self.next_tail.ring_add(amount.get(), cap);
        let end = self.next_tail;
        if start < end {
            (start..end, None)
        } else {
            ((start..cap + 1), Some(0..end))
        }
    }
    #[must_use]
    pub fn dequeue(&mut self, cap: usize) -> Option<usize> {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        if self.is_empty(cap) {
            return None;
        }
        let index = self.head(cap);
        self.prev_head = index;
        Some(index)
    }
    #[must_use]
    pub fn batch_dequeue(
        &mut self,
        amount: usize,
        cap: usize,
    ) -> Option<(core::ops::Range<usize>, Option<core::ops::Range<usize>>)> {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        let amount = self.len(cap).min(amount);
        assert!(amount <= self.len(cap));
        if self.is_empty(cap) {
            return None;
        }
        let start = self.head(cap);
        self.prev_head = self.prev_head.ring_add(amount, cap);
        let end = self.head(cap);
        Some(if start < end {
            (start..end, None)
        } else {
            ((start..cap + 1), Some(0..end))
        })
    }
    #[must_use]
    pub fn as_slices(
        &self,
        cap: usize,
    ) -> Option<(core::ops::Range<usize>, Option<core::ops::Range<usize>>)> {
        #[cfg(debug_assertions)]
        assert_eq!(self.cap, cap);
        if self.is_empty(cap) {
            return None;
        }
        let start = self.head(cap);
        let end = self.next_tail;
        Some(if start < end {
            (start..end, None)
        } else {
            ((start..cap + 1), Some(0..end))
        })
    }
}
#[cfg(not(debug_assertions))]
impl Default for CapQueuePointer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BitQueue {
    pointer: CapQueuePointer,
    set: BitSet,
}
impl BitQueue {
    #[must_use]
    pub fn new(len: usize) -> Self {
        let set_len = len + 1;
        let set = BitSet::new(set_len);
        Self {
            pointer: CapQueuePointer::new(
                #[cfg(debug_assertions)]
                set.capacity().checked_sub(1).unwrap(),
            ),
            set,
        }
    }
    pub fn enqueue(&mut self, value: bool) {
        let index = self.pointer.enqueue(self.capacity());
        match value {
            true => self.set.set(index),
            false => self.set.clear_bit(index),
        }
    }
    pub fn dequeue(&mut self) -> Option<bool> {
        let index = self.pointer.dequeue(self.capacity())?;
        Some(self.set.get(index))
    }
    fn set_index(&self, index: usize) -> usize {
        let head = self.pointer.head(self.capacity());
        head.ring_add(index, self.capacity())
    }
    pub fn get(&self, index: usize) -> bool {
        self.set.get(self.set_index(index))
    }
    pub fn set(&mut self, index: usize, value: bool) {
        let index = self.set_index(index);
        match value {
            true => self.set.set(index),
            false => self.set.clear_bit(index),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = bool> + '_ {
        let head = self.pointer.head(self.capacity());
        (0..self.len()).map(move |i| {
            let i = head.ring_add(i, self.capacity());
            self.set.get(i)
        })
    }
}
impl Capacity for BitQueue {
    fn capacity(&self) -> usize {
        self.set.capacity().checked_sub(1).unwrap()
    }
}
impl Len for BitQueue {
    fn len(&self) -> usize {
        self.pointer.len(self.capacity())
    }
}
impl Clear for BitQueue {
    fn clear(&mut self) {
        #[cfg(not(debug_assertions))]
        {
            self.pointer = CapQueuePointer::new();
        };
        #[cfg(debug_assertions)]
        {
            self.pointer = CapQueuePointer::new(self.pointer.cap());
        };
        self.set.clear();
    }
}

pub type CapVecQueue<T> = CapQueue<Vec<MaybeUninit<T>>, T>;
impl<T> CapVecQueue<T> {
    pub fn new_vec(capacity: usize) -> Self {
        let buf_len = capacity + 1;
        let mut buf = Vec::with_capacity(buf_len);
        buf.extend((0..buf.capacity()).map(|_| MaybeUninit::uninit()));
        Self::new(buf)
    }
}
pub type CapArrayQueue<T, const N: usize> = CapQueue<[MaybeUninit<T>; N], T>;
impl<T, const N: usize> CapArrayQueue<T, N> {
    /// Capacity is actually `N - 1`
    pub fn new_array() -> Self {
        let buf = [const { MaybeUninit::uninit() }; N];
        Self::new(buf)
    }
}

#[derive(Debug, Clone)]
pub struct CapQueue<L: ListMut<MaybeUninit<T>>, T> {
    buf: L,
    item: PhantomData<T>,
    pointer: CapQueuePointer,
}
impl<L, T> CapQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    #[must_use]
    pub fn new(buf: L) -> Self {
        assert!(!buf.is_empty());
        let pointer;
        #[cfg(debug_assertions)]
        {
            pointer = CapQueuePointer::new(buf.len() - 1);
        }
        #[cfg(not(debug_assertions))]
        {
            pointer = CapQueuePointer::new();
        }
        Self {
            buf,
            pointer,
            item: PhantomData,
        }
    }
    pub fn enqueue(&mut self, item: T) {
        let index = self.pointer.enqueue(self.capacity());
        self.buf[index] = MaybeUninit::new(item);
    }
    pub fn batch_enqueue(&mut self, items: &[T])
    where
        T: Copy,
        L: AsSliceMut<MaybeUninit<T>>,
    {
        let Some(items_len) = NonZeroUsize::new(items.len()) else {
            return;
        };
        let (a, b) = self.pointer.batch_enqueue(items_len, self.capacity());
        let a_len = a.clone().len();
        self.buf.as_slice_mut()[a].copy_from_slice(unsafe {
            core::mem::transmute::<&[T], &[MaybeUninit<T>]>(&items[..a_len])
        });
        if let Some(b) = b {
            self.buf.as_slice_mut()[b].copy_from_slice(unsafe {
                core::mem::transmute::<&[T], &[MaybeUninit<T>]>(&items[a_len..])
            });
        }
    }
    pub fn dequeue(&mut self) -> Option<T> {
        let index = self.pointer.dequeue(self.capacity())?;
        let value = &mut self.buf[index];
        let value = core::mem::replace(value, MaybeUninit::uninit());
        Some(unsafe { value.assume_init() })
    }
    pub fn batch_dequeue_iter<'a>(&mut self, amount: usize) -> impl Iterator<Item = &T> + '_
    where
        T: Copy + 'a,
        L: AsSlice<MaybeUninit<T>>,
    {
        let (a, b) = match self.batch_dequeue(amount) {
            None => (&[][..], &[][..]),
            Some((a, b)) => (a, b.unwrap_or(&[])),
        };
        a.iter().chain(b)
    }
    pub fn batch_dequeue_extend<'a>(
        &'a mut self,
        amount: usize,
        extender: &mut impl core::iter::Extend<&'a T>,
    ) where
        T: Copy + 'a,
        L: AsSlice<MaybeUninit<T>>,
    {
        let Some((a, b)) = self.batch_dequeue(amount) else {
            return;
        };
        extender.extend(a.iter());
        if let Some(b) = b {
            extender.extend(b.iter());
        }
    }
    pub fn batch_dequeue(&mut self, amount: usize) -> Option<(&[T], Option<&[T]>)>
    where
        T: Copy,
        L: AsSlice<MaybeUninit<T>>,
    {
        let (a, b) = self.pointer.batch_dequeue(amount, self.capacity())?;
        Some(self.slices(a, b))
    }
    pub fn as_slices(&self) -> Option<(&[T], Option<&[T]>)>
    where
        L: AsSlice<MaybeUninit<T>>,
    {
        let (a, b) = self.pointer.as_slices(self.capacity())?;
        Some(self.slices(a, b))
    }
    fn slices(
        &self,
        a: core::ops::Range<usize>,
        b: Option<core::ops::Range<usize>>,
    ) -> (&[T], Option<&[T]>)
    where
        L: AsSlice<MaybeUninit<T>>,
    {
        let a = unsafe { core::mem::transmute::<&[MaybeUninit<T>], &[T]>(&self.buf.as_slice()[a]) };
        let b = b.map(|b| unsafe {
            core::mem::transmute::<&[MaybeUninit<T>], &[T]>(&self.buf.as_slice()[b])
        });
        (a, b)
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        let head = self.pointer.head(self.capacity());
        (0..self.len()).map(move |i| {
            let i = head.ring_add(i, self.capacity());
            let value = &self.buf[i];
            unsafe { value.assume_init_ref() }
        })
    }
}
impl<L, T> Capacity for CapQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    fn capacity(&self) -> usize {
        self.buf.len().checked_sub(1).unwrap()
    }
}
impl<L, T> Len for CapQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    fn len(&self) -> usize {
        self.pointer.len(self.capacity())
    }
}
impl<L, T> Clear for CapQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    fn clear(&mut self) {
        while let Some(item) = self.dequeue() {
            drop(item);
        }
    }
}
impl<L, T> Drop for CapQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    fn drop(&mut self) {
        while let Some(item) = self.dequeue() {
            drop(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_queue() {
        let mut q = CapArrayQueue::<_, 3>::new_array();
        assert!(q.is_empty());
        q.enqueue(1);
        assert_eq!(q.len(), 1);
        q.enqueue(2);
        assert_eq!(q.len(), 2);
        assert_eq!(q.dequeue().unwrap(), 1);
        assert_eq!(q.len(), 1);
        q.enqueue(3);
        assert_eq!(q.len(), 2);
        assert_eq!(q.clone().iter().copied().collect::<Vec<_>>(), [2, 3]);
        assert_eq!(q.dequeue().unwrap(), 2);
        assert_eq!(q.len(), 1);
        assert_eq!(q.dequeue().unwrap(), 3);
        assert!(q.is_empty());

        for _ in 0..4 {
            q.batch_enqueue(&[1, 2]);
            assert_eq!(q.len(), 2);
            assert_eq!(q.dequeue().unwrap(), 1);
            assert_eq!(q.dequeue().unwrap(), 2);
        }

        for _ in 0..4 {
            q.batch_enqueue(&[1, 2]);
            {
                let (a, b) = q.as_slices().unwrap();
                let mut s: Vec<i32> = vec![];
                s.extend(a);
                if let Some(b) = b {
                    s.extend(b);
                }
                assert_eq!(s, [1, 2]);
            }
            let mut s: Vec<i32> = vec![];
            s.extend(q.batch_dequeue_iter(3));
            assert_eq!(s, [1, 2]);
        }

        for _ in 0..4 {
            q.batch_enqueue(&[]);
            let mut s: Vec<i32> = vec![];
            s.extend(q.batch_dequeue_iter(3));
            assert_eq!(s, []);
        }
    }
    #[test]
    fn test_bit_queue() {
        let mut q = BitQueue::new(2);
        assert!(q.is_empty());
        q.enqueue(false);
        assert_eq!(q.len(), 1);
        q.enqueue(true);
        assert_eq!(q.len(), 2);
        assert!(!q.dequeue().unwrap());
        q.enqueue(true);
        assert_eq!(q.len(), 2);
        assert!(q.dequeue().unwrap());
        assert!(q.dequeue().unwrap());
        assert!(q.dequeue().is_none());
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use std::collections::VecDeque;

    use test::{black_box, Bencher};

    use super::*;

    const CAPACITY: usize = 1 << 10;
    const BATCH_SIZE: usize = CAPACITY / 2;
    type Item = u8;

    #[bench]
    fn bench_vec_deque_drain(bencher: &mut Bencher) {
        let mut q: VecDeque<Item> = VecDeque::with_capacity(CAPACITY);
        let b = batch_buf();
        let mut recv = vec![];
        bencher.iter(|| {
            q.extend(&b);
            recv.extend(q.drain(..));
            black_box(&recv);
            recv.clear();
        });
    }
    #[bench]
    fn bench_vec_deque_slices_clear(bencher: &mut Bencher) {
        let mut q: VecDeque<Item> = VecDeque::with_capacity(CAPACITY);
        let b = batch_buf();
        let mut recv = vec![];
        bencher.iter(|| {
            q.extend(&b);
            let (a, b) = q.as_slices();
            recv.extend(a.iter().copied().chain(b.iter().copied()));
            q.drain(..);
            assert!(!recv.is_empty());
            black_box(&recv);
            recv.clear();
        });
    }
    #[bench]
    fn bench_cap_array_queue_iter(bencher: &mut Bencher) {
        const ARRAY_SIZE: usize = CAPACITY + 1;
        let mut q = CapArrayQueue::<Item, ARRAY_SIZE>::new_array();
        let b = batch_buf();
        let mut recv: Vec<Item> = vec![];
        bencher.iter(|| {
            q.batch_enqueue(&b);
            recv.extend(q.batch_dequeue_iter(b.len()));
            black_box(&recv);
            recv.clear();
        });
    }
    #[bench]
    fn bench_cap_array_queue_extend(bencher: &mut Bencher) {
        const ARRAY_SIZE: usize = CAPACITY + 1;
        let mut q = CapArrayQueue::<Item, ARRAY_SIZE>::new_array();
        let b = batch_buf();
        let mut recv: Vec<Item> = vec![];
        bencher.iter(|| {
            q.batch_enqueue(&b);
            q.batch_dequeue_extend(b.len(), &mut recv);
            black_box(&recv);
            recv.clear();
        });
    }
    #[bench]
    fn bench_cap_vec_queue_iter(bencher: &mut Bencher) {
        let mut q = CapVecQueue::<Item>::new_vec(CAPACITY);
        let b = batch_buf();
        let mut recv: Vec<Item> = vec![];
        bencher.iter(|| {
            q.batch_enqueue(&b);
            recv.extend(q.batch_dequeue_iter(b.len()));
            black_box(&recv);
            recv.clear();
        });
    }

    fn batch_buf() -> Vec<Item> {
        let mut b = vec![];
        b.extend((0..BATCH_SIZE).map(|i| i as Item));
        b
    }
}
