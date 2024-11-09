use std::collections::VecDeque;

use crate::{ops::len::Len, Clear};

#[derive(Debug, Clone)]
pub struct IndexedQueue<T> {
    queue: VecDeque<Option<T>>,
    start: u64,
    count: usize,
}
impl<T> IndexedQueue<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            start: 0,
            count: 0,
        }
    }

    #[must_use]
    pub fn enqueue(&mut self, value: T) -> QueueIndex {
        let new_index = self.queue.len();
        self.queue.push_back(Some(value));
        self.count += 1;
        QueueIndex {
            start: self.start,
            offset: new_index,
        }
    }
    pub fn dequeue(&mut self) -> Option<T> {
        while let Some(entry) = self.queue.pop_front() {
            self.start = self.start.wrapping_add(1);
            let Some(value) = entry else {
                continue;
            };
            self.count -= 1;
            return Some(value);
        }
        None
    }

    #[must_use]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        loop {
            let Some(front) = self.queue.front() else {
                break;
            };
            if front.is_some() {
                break;
            }
            self.queue.pop_front();
            self.start = self.start.wrapping_add(1);
        }
        self.queue.front_mut().map(|entry| entry.as_mut().unwrap())
    }
    #[must_use]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        loop {
            let Some(back) = self.queue.back() else {
                break;
            };
            if back.is_some() {
                break;
            }
            self.queue.pop_back();
        }
        self.queue.back_mut().map(|entry| entry.as_mut().unwrap())
    }

    pub fn remove(&mut self, index: QueueIndex) -> Option<T> {
        let index = self.local_index(index)?;
        let entry = self.queue.get_mut(index).unwrap();
        let value = entry.take();
        if value.is_some() {
            self.count -= 1;
        }
        value
    }
    #[must_use]
    pub fn get(&self, index: QueueIndex) -> Option<&T> {
        let index = self.local_index(index)?;
        let entry = self.queue.get(index).unwrap();
        entry.as_ref()
    }
    #[must_use]
    pub fn get_mut(&mut self, index: QueueIndex) -> Option<&mut T> {
        let index = self.local_index(index)?;
        let entry = self.queue.get_mut(index).unwrap();
        entry.as_mut()
    }
    #[must_use]
    pub fn local_index(&self, index: QueueIndex) -> Option<usize> {
        let start_diff = self.start.wrapping_sub(index.start);
        let start_diff = usize::try_from(start_diff).ok()?;
        let local_index = index.offset.checked_sub(start_diff)?;
        if local_index < self.queue.len() {
            Some(local_index)
        } else {
            None
        }
    }

    pub fn trim(&mut self) {
        let _ = self.front_mut();
        let _ = self.back_mut();
    }
}
impl<T> Default for IndexedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Len for IndexedQueue<T> {
    fn len(&self) -> usize {
        self.count
    }
}
impl<T> Clear for IndexedQueue<T> {
    fn clear(&mut self) {
        let queue_len = self.queue.len();
        let queue_len = u64::try_from(queue_len).unwrap();
        let new_start = self.start.wrapping_add(queue_len);
        self.start = new_start;
        self.count = 0;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct QueueIndex {
    start: u64,
    offset: usize,
}
impl QueueIndex {
    #[must_use]
    fn canonical(&self) -> u64 {
        let offset = u64::try_from(self.offset).unwrap();
        self.start.wrapping_add(offset)
    }
}
impl PartialEq for QueueIndex {
    fn eq(&self, other: &Self) -> bool {
        self.canonical() == other.canonical()
    }
}
impl Eq for QueueIndex {}

#[cfg(test)]
mod tests {
    use crate::ops::len::LenExt;

    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        let mut queue = IndexedQueue::new();
        let index_0 = queue.enqueue(0);
        {
            assert_eq!(
                index_0,
                QueueIndex {
                    start: 0,
                    offset: 0
                }
            );
            assert_eq!(*queue.get(index_0).unwrap(), 0);
        }
        let index_1 = queue.enqueue(1);
        {
            assert_eq!(
                index_1,
                QueueIndex {
                    start: 0,
                    offset: 1
                }
            );
            assert_eq!(*queue.get(index_1).unwrap(), 1);
        }
        assert_eq!(queue.dequeue().unwrap(), 0);
        {
            assert_eq!(queue.get(index_0), None);
            assert_eq!(*queue.get(index_1).unwrap(), 1);
        }
        assert_eq!(queue.dequeue().unwrap(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut queue = IndexedQueue::new();
        let index_0 = queue.enqueue(0);
        let index_1 = queue.enqueue(1);
        let index_2 = queue.enqueue(2);
        queue.remove(index_0).unwrap();
        assert_eq!(queue.len(), 2);
        queue.remove(index_1).unwrap();
        assert_eq!(queue.len(), 1);
        assert_eq!(*queue.front_mut().unwrap(), 2);
        assert_eq!(queue.dequeue().unwrap(), 2);
        assert!(queue.get(index_2).is_none());
        assert!(queue.is_empty());
    }
}
