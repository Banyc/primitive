use core::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use crate::{ops::len::Len, ops::opt_cmp::MinNoneOptCmp, Clear};

#[derive(Debug, Clone)]
pub struct OrdQueue<T> {
    min_heap: BinaryHeap<Reverse<T>>,
    linear: VecDeque<T>,
}
impl<T: Ord> OrdQueue<T> {
    pub fn new() -> Self {
        Self {
            min_heap: BinaryHeap::new(),
            linear: VecDeque::new(),
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        Some(match self.min_head_location()? {
            Location::MinHeap => self.min_heap.pop().unwrap().0,
            Location::Linear => self.linear.pop_front().unwrap(),
        })
    }
    pub fn peek(&self) -> Option<&T> {
        Some(match self.min_head_location()? {
            Location::MinHeap => &self.min_heap.peek().unwrap().0,
            Location::Linear => self.linear.front().unwrap(),
        })
    }
    fn min_head_location(&self) -> Option<Location> {
        let min_heap_head = self.min_heap.peek().map(|Reverse(value)| value);
        let linear_head = self.linear.front();
        let (min_heap_head, linear_head) = match (min_heap_head, linear_head) {
            (None, None) => return None,
            (None, Some(_)) => {
                return Some(Location::Linear);
            }
            (Some(_), None) => {
                return Some(Location::MinHeap);
            }
            (Some(min_heap_head), Some(linear_head)) => (min_heap_head, linear_head),
        };
        match min_heap_head.cmp(linear_head) {
            core::cmp::Ordering::Less => Some(Location::MinHeap),
            core::cmp::Ordering::Equal | core::cmp::Ordering::Greater => Some(Location::Linear),
        }
    }
    pub fn insert(&mut self, value: T) {
        let linear_back = self.linear.back();
        if MinNoneOptCmp(linear_back) <= MinNoneOptCmp(Some(&value)) {
            self.linear.push_back(value);
            return;
        }
        self.min_heap.push(Reverse(value));
    }
}
impl<T: Ord> Default for OrdQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Len for OrdQueue<T> {
    fn len(&self) -> usize {
        self.linear.len() + self.min_heap.len()
    }
}
impl<T> Clear for OrdQueue<T> {
    fn clear(&mut self) {
        self.linear.clear();
        self.min_heap.clear();
    }
}

enum Location {
    MinHeap,
    Linear,
}

#[cfg(test)]
mod tests {
    use crate::ops::len::LenExt;

    use super::*;

    #[test]
    fn test_ordered_queue() {
        let mut q = OrdQueue::new();
        assert!(q.pop().is_none());
        q.insert(3);
        assert_eq!(q.peek().unwrap(), (&3));
        assert_eq!(q.len(), 1);
        q.insert(2);
        assert_eq!(q.peek().unwrap(), (&2));
        assert_eq!(q.len(), 2);
        q.insert(3);
        assert_eq!(q.peek().unwrap(), (&2));
        assert_eq!(q.len(), 3);
        assert_eq!(q.pop().unwrap(), (2));
        assert_eq!(q.pop().unwrap(), (3));
        assert_eq!(q.pop().unwrap(), (3));
        assert!(q.pop().is_none());
        assert!(q.is_empty());
    }
}
