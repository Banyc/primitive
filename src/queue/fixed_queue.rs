use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{
    ops::{
        len::{Capacity, Len, LenExt},
        list::ListMut,
        ring::RingSpace,
    },
    set::bit_set::BitSet,
    Clear,
};

#[derive(Debug, Clone, Copy)]
pub struct FixedQueuePointer {
    prev_head: usize,
    next_tail: usize,
}
impl FixedQueuePointer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            prev_head: 0,
            next_tail: 1,
        }
    }
    #[must_use]
    pub fn head(&self, cap: usize) -> usize {
        self.prev_head.ring_add(1, cap)
    }
    #[must_use]
    pub fn len(&self, cap: usize) -> usize {
        let dist = self.next_tail.ring_sub(self.prev_head, cap);
        dist.checked_sub(1).unwrap_or(cap)
    }
    #[must_use]
    pub fn enqueue(&mut self, cap: usize) -> usize {
        if self.prev_head == self.next_tail {
            panic!("out of buffer space");
        }
        let index = self.next_tail;
        self.next_tail = self.next_tail.ring_add(1, cap);
        index
    }
    #[must_use]
    pub fn dequeue(&mut self, cap: usize) -> Option<usize> {
        let is_empty = self.len(cap) == 0;
        if is_empty {
            return None;
        }
        let index = self.head(cap);
        self.prev_head = index;
        Some(index)
    }
}
impl Default for FixedQueuePointer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BitQueue {
    pointer: FixedQueuePointer,
    set: BitSet,
}
impl BitQueue {
    #[must_use]
    pub fn new(len: usize) -> Self {
        let set_len = len + 1;
        Self {
            pointer: FixedQueuePointer::new(),
            set: BitSet::new(set_len),
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
        self.pointer = FixedQueuePointer::new();
        self.set.clear();
    }
}

#[derive(Debug)]
pub struct FixedQueue<L: ListMut<MaybeUninit<T>>, T> {
    buf: L,
    item: PhantomData<T>,
    pointer: FixedQueuePointer,
}
impl<L, T> FixedQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    #[must_use]
    pub fn new(buf: L) -> Self {
        assert!(!buf.is_empty());
        Self {
            buf,
            pointer: FixedQueuePointer::new(),
            item: PhantomData,
        }
    }
    pub fn enqueue(&mut self, item: T) {
        let index = self.pointer.enqueue(self.capacity());
        self.buf[index] = MaybeUninit::new(item);
    }
    pub fn dequeue(&mut self) -> Option<T> {
        let index = self.pointer.dequeue(self.capacity())?;
        let value = &mut self.buf[index];
        let value = core::mem::replace(value, MaybeUninit::uninit());
        Some(unsafe { value.assume_init() })
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
impl<L, T> Capacity for FixedQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    fn capacity(&self) -> usize {
        self.buf.len().checked_sub(1).unwrap()
    }
}
impl<L, T> Len for FixedQueue<L, T>
where
    L: ListMut<MaybeUninit<T>>,
{
    fn len(&self) -> usize {
        self.pointer.len(self.capacity())
    }
}
impl<L, T> Clone for FixedQueue<L, T>
where
    L: ListMut<MaybeUninit<T>> + Clone,
    T: Clone,
{
    fn clone(&self) -> Self {
        let buf = self.buf.clone();
        let mut new = Self::new(buf);
        for item in self.iter() {
            new.enqueue(item.clone());
        }
        new
    }
}
impl<L, T> Drop for FixedQueue<L, T>
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
    fn test_fixed_queue() {
        let mut q = FixedQueue::new([MaybeUninit::uninit(); 3]);
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
