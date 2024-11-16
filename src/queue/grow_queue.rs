use crate::ops::{
    clear::Clear,
    len::{Capacity, Full, Len},
};

use super::cap_queue::CapVecQueue;

const START_UP_SIZE: usize = 16;

#[derive(Debug)]
pub struct GrowQueue<T> {
    vec_queue: Option<CapVecQueue<T>>,
}
impl<T> GrowQueue<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self { vec_queue: None }
    }
    #[must_use]
    fn ensure_primed(&mut self) -> &mut CapVecQueue<T> {
        if self.vec_queue.is_some() {
            return self.vec_queue.as_mut().unwrap();
        }
        self.vec_queue = Some(CapVecQueue::new_vec(START_UP_SIZE));
        self.vec_queue.as_mut().unwrap()
    }
    #[must_use]
    fn exp_grow(&mut self) -> &mut CapVecQueue<T> {
        let vec_queue = self.vec_queue.as_mut().unwrap();
        let mut new = CapVecQueue::new_vec(vec_queue.capacity() * 2);
        while let Some(item) = vec_queue.dequeue() {
            new.enqueue(item);
        }
        self.vec_queue = Some(new);
        self.vec_queue.as_mut().unwrap()
    }
    #[must_use]
    fn exp_grow_copy(&mut self, cap_at_least: usize) -> &mut CapVecQueue<T>
    where
        T: Copy,
    {
        let vec_queue = self.vec_queue.as_mut().unwrap();
        let mut new_cap = vec_queue.capacity();
        loop {
            if cap_at_least <= new_cap {
                break;
            }
            new_cap *= 2;
        }
        let mut new = CapVecQueue::new_vec(new_cap);
        if let Some((a, b)) = vec_queue.as_slices() {
            new.batch_enqueue(a);
            if let Some(b) = b {
                new.batch_enqueue(b);
            }
        }
        self.vec_queue = Some(new);
        self.vec_queue.as_mut().unwrap()
    }
    pub fn enqueue(&mut self, item: T) {
        let vec_queue = self.ensure_primed();
        let vec_queue = if vec_queue.is_full() {
            self.exp_grow()
        } else {
            vec_queue
        };
        vec_queue.enqueue(item);
    }
    pub fn dequeue(&mut self) -> Option<T> {
        let vec_queue = self.vec_queue.as_mut()?;
        vec_queue.dequeue()
    }
    pub fn batch_enqueue(&mut self, items: &[T])
    where
        T: Copy,
    {
        let vec_queue = self.ensure_primed();
        let cap_at_least = vec_queue.len() + items.len();
        let vec_queue = if vec_queue.capacity() < cap_at_least {
            self.exp_grow_copy(cap_at_least)
        } else {
            vec_queue
        };
        vec_queue.batch_enqueue(items);
    }
    pub fn batch_dequeue_extend<'a>(
        &'a mut self,
        amount: usize,
        extender: &mut impl core::iter::Extend<&'a T>,
    ) where
        T: Copy + 'a,
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
    {
        let vec_queue = self.vec_queue.as_mut()?;
        vec_queue.batch_dequeue(amount)
    }
    #[must_use]
    pub fn as_slices(&self) -> Option<(&[T], Option<&[T]>)> {
        self.vec_queue.as_ref()?.as_slices()
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.vec_queue.iter().flat_map(|q| q.iter())
    }
}
impl<T> Default for GrowQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Len for GrowQueue<T> {
    fn len(&self) -> usize {
        let Some(vec_queue) = &self.vec_queue else {
            return 0;
        };
        vec_queue.len()
    }
}
impl<T> Clear for GrowQueue<T> {
    fn clear(&mut self) {
        let Some(vec_queue) = &mut self.vec_queue else {
            return;
        };
        vec_queue.clear();
    }
}
impl<T: Copy> Clone for GrowQueue<T> {
    fn clone(&self) -> Self {
        Self {
            vec_queue: self.vec_queue.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grow_queue() {
        let mut q = GrowQueue::new();
        assert_eq!(q.len(), 0);
        q.enqueue(0);
        assert_eq!(q.len(), 1);
        assert_eq!(q.iter().copied().collect::<Vec<_>>(), [0]);
        q.batch_enqueue(&(0..START_UP_SIZE).map(|x| x + 1).collect::<Vec<_>>());
        assert_eq!(q.len(), START_UP_SIZE + 1);
        assert_eq!(
            q.iter().copied().collect::<Vec<_>>(),
            (0..START_UP_SIZE + 1).collect::<Vec<_>>()
        );
    }
}
