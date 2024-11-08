use core::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use crate::{ops::opt_cmp::MinNoneOptCmp, Clear, Len};

#[derive(Debug, Clone)]
pub struct OrderedQueue<K, V> {
    min_heap: BinaryHeap<Reverse<Entry<K, V>>>,
    linear: VecDeque<Entry<K, V>>,
}
impl<K: Ord, V> OrderedQueue<K, V> {
    pub fn new() -> Self {
        Self {
            min_heap: BinaryHeap::new(),
            linear: VecDeque::new(),
        }
    }
    pub fn insert(&mut self, key: K, value: V) {
        let entry = Entry { key, value };
        let linear_back = self.linear.back().map(|entry| &entry.key);
        if MinNoneOptCmp(linear_back) <= MinNoneOptCmp(Some(&entry.key)) {
            self.linear.push_back(entry);
            return;
        }
        self.min_heap.push(Reverse(entry));
    }
    pub fn pop(&mut self) -> Option<(K, V)> {
        Some(match self.min_head_location()? {
            Location::MinHeap => self.min_heap.pop().unwrap().0.into_flatten(),
            Location::Linear => self.linear.pop_front().unwrap().into_flatten(),
        })
    }
    pub fn peak(&self) -> Option<(&K, &V)> {
        Some(match self.min_head_location()? {
            Location::MinHeap => self.min_heap.peek().unwrap().0.flatten(),
            Location::Linear => self.linear.front().unwrap().flatten(),
        })
    }
    fn min_head_location(&self) -> Option<Location> {
        let min_heap_head = self.min_heap.peek().map(|Reverse(entry)| &entry.key);
        let linear_head = self.linear.front().map(|entry| &entry.key);
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
}
impl<K: Ord, V> Default for OrderedQueue<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V> Len for OrderedQueue<K, V> {
    fn len(&self) -> usize {
        self.linear.len() + self.min_heap.len()
    }
}
impl<K, V> Clear for OrderedQueue<K, V> {
    fn clear(&mut self) {
        self.linear.clear();
        self.min_heap.clear();
    }
}

enum Location {
    MinHeap,
    Linear,
}

#[derive(Debug, Clone)]
struct Entry<K, V> {
    pub key: K,
    pub value: V,
}
impl<K, V> Entry<K, V> {
    pub fn into_flatten(self) -> (K, V) {
        (self.key, self.value)
    }
    pub fn flatten(&self) -> (&K, &V) {
        (&self.key, &self.value)
    }
}
impl<K: PartialEq, V> PartialEq for Entry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<K: Eq, V> Eq for Entry<K, V> {}
impl<K: PartialOrd, V> PartialOrd for Entry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}
impl<K: Ord, V> Ord for Entry<K, V> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

#[cfg(test)]
mod tests {
    use crate::LenExt;

    use super::*;

    #[test]
    fn test_ordered_queue() {
        let mut q = OrderedQueue::new();
        assert!(q.pop().is_none());
        q.insert(3, 3);
        assert_eq!(q.peak().unwrap(), (&3, &3));
        assert_eq!(q.len(), 1);
        q.insert(2, 2);
        assert_eq!(q.peak().unwrap(), (&2, &2));
        assert_eq!(q.len(), 2);
        q.insert(3, 3);
        assert_eq!(q.peak().unwrap(), (&2, &2));
        assert_eq!(q.len(), 3);
        assert_eq!(q.pop().unwrap(), (2, 2));
        assert_eq!(q.pop().unwrap(), (3, 3));
        assert_eq!(q.pop().unwrap(), (3, 3));
        assert!(q.pop().is_none());
        assert!(q.is_empty());
    }
}
