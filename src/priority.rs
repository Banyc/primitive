use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use thiserror::Error;

/// The lower the priority number, the higher the priority
#[derive(Debug, Clone)]
pub struct Queue<T, const P: usize> {
    cap: usize,
    queues: [VecDeque<Entry<T>>; P],
    timeout: Option<Duration>,
}
impl<T, const P: usize> Queue<T, P> {
    pub fn new(cap: usize, timeout: Option<Duration>) -> Self {
        let queues = (0..P)
            .map(|_| VecDeque::new())
            .collect::<Vec<VecDeque<Entry<T>>>>()
            .try_into()
            .unwrap_or_else(|_| unreachable!());
        Self {
            cap,
            queues,
            timeout,
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }
    pub fn len(&self) -> usize {
        self.queues.iter().map(|q| q.len()).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn pop_one_timed_out(&mut self, now: Instant) -> Option<Entry<T>> {
        let timeout = self.timeout?;
        for queue in self.queues.iter_mut().rev() {
            let Some(item) = queue.front() else {
                continue;
            };
            if item.is_timed_out(timeout, now) {
                return queue.pop_front();
            }
        }
        None
    }

    /// The lower the `pri`, the higher the priority
    pub fn push(
        &mut self,
        item: T,
        pri: usize,
        now: Instant,
    ) -> Result<Option<Entry<T>>, PushError<T>> {
        assert!(pri < P);
        let mut timed_out = None;
        if self.len() == self.capacity() {
            let Some(item) = self.pop_one_timed_out(now) else {
                return Err(PushError::Cap(item));
            };
            timed_out = Some(item);
        }
        let entry = Entry::new(item, now);
        self.queues[pri].push_back(entry);
        Ok(timed_out)
    }

    pub fn fifo_pop(&mut self) -> Option<Entry<T>> {
        for queue in self.queues.iter_mut() {
            if let Some(item) = queue.pop_front() {
                return Some(item);
            }
        }
        None
    }
    pub fn lifo_pop(&mut self) -> Option<Entry<T>> {
        for queue in self.queues.iter_mut() {
            if let Some(item) = queue.pop_back() {
                return Some(item);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Error)]
pub enum PushError<T> {
    #[error("reached full capacity")]
    Cap(T),
}

#[derive(Debug, Clone)]
pub struct Entry<T> {
    item: T,
    insert: Instant,
}
impl<T> Entry<T> {
    pub fn new(item: T, insert: Instant) -> Self {
        Self { item, insert }
    }

    pub fn insertion_time(&self) -> Instant {
        self.insert
    }
    pub fn is_timed_out(&self, timeout: Duration, now: Instant) -> bool {
        timeout < now.duration_since(self.insertion_time())
    }

    pub fn into_item(self) -> T {
        self.item
    }
}
