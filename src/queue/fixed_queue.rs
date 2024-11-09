use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{
    ops::{ring::RingSpace, seq::SeqMut},
    Capacity, Len, LenExt,
};

#[derive(Debug)]
pub struct FixedQueue<S: SeqMut<MaybeUninit<T>>, T> {
    buf: S,
    item: PhantomData<T>,
    prev_head: usize,
    next_tail: usize,
}
impl<S, T> FixedQueue<S, T>
where
    S: SeqMut<MaybeUninit<T>>,
{
    #[must_use]
    pub fn new(buf: S) -> Self {
        assert!(!buf.as_slice().is_empty());
        Self {
            buf,
            prev_head: 0,
            next_tail: 1,
            item: PhantomData,
        }
    }
    pub fn enqueue(&mut self, item: T) {
        if self.prev_head == self.next_tail {
            panic!("out of buffer space");
        }
        self.buf.as_slice_mut()[self.next_tail] = MaybeUninit::new(item);
        self.next_tail = self.next_tail.ring_add(1, self.capacity());
    }
    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let head = self.head();
        self.prev_head = head;
        let value = &mut self.buf.as_slice_mut()[head];
        let value = core::mem::replace(value, MaybeUninit::uninit());
        Some(unsafe { value.assume_init() })
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        let head = self.head();
        (0..self.len()).map(move |i| {
            let i = head.ring_add(i, self.capacity());
            let value = &self.buf.as_slice()[i];
            unsafe { value.assume_init_ref() }
        })
    }
    fn head(&self) -> usize {
        self.prev_head.ring_add(1, self.capacity())
    }
}
impl<S, T> Capacity for FixedQueue<S, T>
where
    S: SeqMut<MaybeUninit<T>>,
{
    fn capacity(&self) -> usize {
        self.buf.as_slice().len().checked_sub(1).unwrap()
    }
}
impl<S, T> Len for FixedQueue<S, T>
where
    S: SeqMut<MaybeUninit<T>>,
{
    fn len(&self) -> usize {
        let capacity = self.capacity();
        let dist = self.next_tail.ring_sub(self.prev_head, capacity);
        dist.checked_sub(1).unwrap_or(capacity)
    }
}
impl<S, T> Clone for FixedQueue<S, T>
where
    S: SeqMut<MaybeUninit<T>> + Clone,
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
impl<S, T> Drop for FixedQueue<S, T>
where
    S: SeqMut<MaybeUninit<T>>,
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
}
