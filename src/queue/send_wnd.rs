use std::collections::VecDeque;

use num_traits::{CheckedAdd, CheckedSub, NumCast, One};

use crate::ops::len::Len;

#[derive(Debug, Clone)]
pub struct SendWnd<K, V> {
    start: Option<K>,
    next: Option<K>,
    queue: VecDeque<V>,
}
impl<K, V> SendWnd<K, V> {
    pub fn new(next: K) -> Self {
        Self {
            start: None,
            next: Some(next),
            queue: VecDeque::new(),
        }
    }
    pub fn clear(&mut self, next: K) {
        self.start = None;
        self.next = Some(next);
        self.queue.clear();
    }
    pub fn start(&self) -> Option<&K> {
        self.start.as_ref()
    }
    pub fn next(&self) -> Option<&K> {
        self.next.as_ref()
    }
}
impl<K, V> SendWnd<K, V>
where
    K: CheckedAdd + One + Clone + Eq,
{
    pub fn push(&mut self, value: V) {
        let next = self.next.as_ref().unwrap();
        if self.start.is_none() {
            self.start = Some(next.clone());
        }
        self.next = next.checked_add(&K::one());
        self.queue.push_back(value);
    }
    pub fn pop(&mut self) -> Option<V> {
        let start = self.start.as_ref()?;
        self.start = start.checked_add(&K::one());
        if self.start == self.next {
            self.start = None;
        }
        Some(self.queue.pop_front().unwrap())
    }
}
impl<K, V> SendWnd<K, V>
where
    K: CheckedSub + CheckedAdd + One + NumCast + Clone + Eq,
{
    fn queue_index(&self, key: &K) -> Option<usize> {
        let start = self.start.as_ref()?;
        key.checked_sub(start)?.to_usize()
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.queue_index(key)?;
        self.queue.get(index)
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = self.queue_index(key)?;
        self.queue.get_mut(index)
    }
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.queue.iter().enumerate().map(|(i, value)| {
            let start = self.start.as_ref().unwrap();
            let key = start.checked_add(&K::from(i).unwrap()).unwrap();
            (key, value)
        })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.queue.iter_mut().enumerate().map(|(i, value)| {
            let start = self.start.as_ref().unwrap();
            let key = start.checked_add(&K::from(i).unwrap()).unwrap();
            (key, value)
        })
    }
}
impl<K, V> Len for SendWnd<K, V> {
    fn len(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_wnd() {
        let mut w: SendWnd<usize, usize> = SendWnd::new(0);
        assert!(w.pop().is_none());
        assert_eq!(*w.next().unwrap(), 0);
        w.push(0);
        assert_eq!(*w.get(&0).unwrap(), 0);
        assert_eq!(*w.next().unwrap(), 1);
        w.push(1);
        assert_eq!(w.pop().unwrap(), 0);
        assert_eq!(*w.get(&1).unwrap(), 1);
        assert_eq!(w.pop().unwrap(), 1);
        assert!(w.pop().is_none());
    }
}
