use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct IndexedQueue<T> {
    queue: VecDeque<Option<T>>,
    start: u64,
    len: usize,
}
impl<T> IndexedQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            start: 0,
            len: 0,
        }
    }
    pub fn clear(&mut self) {
        let queue_len = self.queue.len();
        let queue_len = u64::try_from(queue_len).unwrap();
        let new_start = self.start.wrapping_add(queue_len);
        self.start = new_start;
        self.len = 0;
    }

    pub fn enqueue(&mut self, value: T) -> QueueIndex {
        let new_index = self.queue.len();
        self.queue.push_back(Some(value));
        self.len += 1;
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
            self.len -= 1;
            return Some(value);
        }
        None
    }

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

    pub fn remove(&mut self, index: QueueIndex) -> Option<T> {
        let index = self.local_index(index)?;
        let entry = self.queue.get_mut(index).unwrap();
        let value = entry.take();
        if value.is_some() {
            self.len -= 1;
        }
        value
    }
    pub fn get(&self, index: QueueIndex) -> Option<&T> {
        let index = self.local_index(index)?;
        let entry = self.queue.get(index).unwrap();
        entry.as_ref()
    }
    pub fn get_mut(&mut self, index: QueueIndex) -> Option<&mut T> {
        let index = self.local_index(index)?;
        let entry = self.queue.get_mut(index).unwrap();
        entry.as_mut()
    }
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

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T> Default for IndexedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueIndex {
    start: u64,
    offset: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexed_queue() {
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
}
