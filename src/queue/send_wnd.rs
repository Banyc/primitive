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
    #[must_use]
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
    #[must_use]
    pub fn start(&self) -> Option<&K> {
        self.start.as_ref()
    }
    #[must_use]
    pub fn next(&self) -> Option<&K> {
        self.next.as_ref()
    }
}
impl<K, U> SendWnd<K, Option<U>>
where
    K: CheckedSub + CheckedAdd + One + NumCast + Clone + Eq,
{
    pub fn pop_none(&mut self) {
        loop {
            let Some((_, v)) = self.iter().nth(0) else {
                break;
            };
            if v.is_some() {
                break;
            }
            self.pop().unwrap();
        }
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
    #[must_use]
    fn queue_index(&self, key: &K) -> Option<usize> {
        let start = self.start.as_ref()?;
        key.checked_sub(start)?.to_usize()
    }
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.queue_index(key)?;
        self.queue.get(index)
    }
    #[must_use]
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
    use crate::ops::len::LenExt;

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

    #[test]
    fn test_send_wnd_pop_none() {
        let mut w: SendWnd<usize, Option<usize>> = SendWnd::new(0);
        w.push(Some(0));
        w.push(Some(1));
        w.push(Some(2));
        *w.get_mut(&1).unwrap() = None;
        *w.get_mut(&2).unwrap() = None;
        assert_eq!(w.len(), 3);
        w.pop_none();
        assert_eq!(w.len(), 3);
        assert_eq!(w.pop().unwrap(), Some(0));
        w.pop_none();
        assert!(w.is_empty());

        let mut w: SendWnd<usize, Option<usize>> = SendWnd::new(0);
        w.push(Some(0));
        w.push(Some(1));
        w.push(Some(2));
        *w.get_mut(&1).unwrap() = None;
        assert_eq!(w.len(), 3);
        w.pop_none();
        assert_eq!(w.len(), 3);
        assert_eq!(w.pop().unwrap(), Some(0));
        w.pop_none();
        assert_eq!(w.len(), 1);
        assert_eq!(*w.get(&2).unwrap(), Some(2));
    }
}
