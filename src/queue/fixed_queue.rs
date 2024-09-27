use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{
    seq::{Seq, SeqMut},
    wrap::RingSpace,
    Capacity, Len, LenExt,
};

#[derive(Debug, Clone, Copy)]
pub struct FixedQueue<S, T> {
    buf: S,
    item: PhantomData<T>,
    prev_head: usize,
    next_tail: usize,
}
impl<S, T> FixedQueue<S, T> {
    pub fn into_buffer(self) -> S {
        self.buf
    }
}
impl<S, T> FixedQueue<S, T>
where
    S: Seq<MaybeUninit<T>>,
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
}
impl<S, T> Capacity for FixedQueue<S, T>
where
    S: Seq<MaybeUninit<T>>,
{
    fn capacity(&self) -> usize {
        self.buf.as_slice().len().checked_sub(1).unwrap()
    }
}
impl<S, T> Len for FixedQueue<S, T>
where
    S: Seq<MaybeUninit<T>>,
{
    fn len(&self) -> usize {
        let capacity = self.capacity();
        let dist = self.next_tail.ring_sub(self.prev_head, capacity);
        dist.checked_sub(1).unwrap_or(capacity)
    }
}
impl<S, T> FixedQueue<S, T>
where
    S: SeqMut<MaybeUninit<T>>,
{
    pub fn enqueue(&mut self, item: T) {
        if self.prev_head == self.next_tail {
            panic!("out of buffer space");
        }
        self.buf.as_slice_mut()[self.next_tail] = MaybeUninit::new(item);
        self.next_tail = self.next_tail.ring_add(1, self.capacity());
    }
    pub fn dequeue(&mut self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let head = self.prev_head.ring_add(1, self.capacity());
        self.prev_head = head;
        Some(unsafe { self.buf.as_slice().get(head).unwrap().assume_init_ref() })
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
        assert_eq!(*q.dequeue().unwrap(), 1);
        assert_eq!(q.len(), 1);
        q.enqueue(3);
        assert_eq!(q.len(), 2);
        assert_eq!(*q.dequeue().unwrap(), 2);
        assert_eq!(q.len(), 1);
        assert_eq!(*q.dequeue().unwrap(), 3);
        assert!(q.is_empty());
    }
}
