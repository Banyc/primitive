use core::{hash::Hash, ops::ControlFlow};
use std::collections::{BTreeMap, HashSet};

use num_traits::{CheckedAdd, One};

use crate::{map::MapInsert, ops::len::Len, queue::ordered_queue::OrderedQueue, Clear};

#[derive(Debug, Clone)]
pub struct SeqQueue<K, V> {
    queue: OrderedQueue<K, V>,
    next: Option<K>,
    keys: Option<HashSet<K>>,
}
impl<K, V> SeqQueue<K, V> {
    #[must_use]
    pub fn next(&self) -> Option<&K> {
        self.next.as_ref()
    }
}
impl<K, V> SeqQueue<K, V>
where
    K: Ord,
{
    /// Slower than [`BTreeSeqQueue`]
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: OrderedQueue::new(),
            next: None,
            keys: Some(HashSet::new()),
        }
    }
    /// No check on duplicate on [`Self::insert()`]
    #[must_use]
    pub fn new_unstable() -> Self {
        Self {
            queue: OrderedQueue::new(),
            next: None,
            keys: None,
        }
    }
}
impl<K, V> SeqQueue<K, V>
where
    K: Ord + Hash,
{
    pub fn set_next(&mut self, next: K, mut stale: impl FnMut((K, V))) {
        loop {
            let Some((head, _)) = self.queue.peak() else {
                break;
            };
            if next <= *head {
                break;
            }
            if let Some(keys) = &mut self.keys {
                assert!(keys.remove(head));
            }
            stale(self.queue.pop().unwrap());
        }
        self.next = Some(next);
    }
}
impl<K, V> SeqQueue<K, V>
where
    K: Ord + CheckedAdd + One + Clone + Hash,
{
    #[must_use]
    pub fn peek(&self) -> Option<(&K, &V)> {
        let next = self.next()?;
        let (k, v) = self.queue.peak()?;
        if k != next {
            return None;
        }
        Some((k, v))
    }
    #[must_use]
    pub fn pop(&mut self, waste: impl FnMut((K, V))) -> Option<(K, V)> {
        let _ = self.peek()?;
        let (k, v) = self.queue.pop().unwrap();
        if let Some(keys) = &mut self.keys {
            assert!(keys.remove(&k));
        } else {
            self.remove_head(waste);
        }
        self.next = self.next().unwrap().checked_add(&K::one());
        Some((k, v))
    }
    fn remove_head(&mut self, mut waste: impl FnMut((K, V))) {
        let Some(next) = self.next.as_ref() else {
            return;
        };
        while let Some((k, _)) = self.queue.peak() {
            if k != next {
                break;
            }
            waste(self.queue.pop().unwrap());
        }
    }
    #[must_use]
    pub fn insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) -> SeqInsertResult {
        let case = insert_case(self.next(), &key);
        match case {
            SeqInsertResult::Stalled | SeqInsertResult::InOrder | SeqInsertResult::OutOfOrder => {
                self.force_insert(key, value, &mut waste);
            }
            SeqInsertResult::Stale => {
                waste((key, value));
            }
        }
        case
    }
    /// Return the input if `key` is [`Self::next()`]
    #[must_use]
    pub fn insert_pop(
        &mut self,
        key: K,
        value: V,
        mut waste: impl FnMut((K, V)),
    ) -> SeqInsertPopResult<K, V> {
        let case = insert_case(self.next(), &key);
        match case {
            SeqInsertResult::Stalled => {
                self.force_insert(key, value, &mut waste);
                SeqInsertPopResult::Stalled
            }
            SeqInsertResult::Stale => {
                waste((key, value));
                SeqInsertPopResult::Stale
            }
            SeqInsertResult::InOrder => {
                if self.pop(waste).is_none() {
                    self.next = self.next().unwrap().checked_add(&K::one());
                }
                SeqInsertPopResult::InOrder((key, value))
            }
            SeqInsertResult::OutOfOrder => {
                self.force_insert(key, value, &mut waste);
                SeqInsertPopResult::OutOfOrder
            }
        }
    }
    fn force_insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) {
        if let Some(keys) = &mut self.keys {
            if keys.contains(&key) {
                waste((key, value));
                return;
            }
            keys.insert(key.clone());
        }
        self.queue.insert(key, value);
    }
    pub fn insert_pop_all<O>(
        &mut self,
        key: K,
        value: V,
        mut waste: impl FnMut((K, V)),
        mut read: impl FnMut((K, V)) -> ControlFlow<O>,
    ) -> Option<O> {
        let (k, v) = self.insert_pop(key, value, &mut waste).into_in_order()?;
        let ctrl = read((k, v));
        if let ControlFlow::Break(o) = ctrl {
            return Some(o);
        }
        while let Some((k, v)) = self.pop(&mut waste) {
            let ctrl = read((k, v));
            if let ControlFlow::Break(o) = ctrl {
                return Some(o);
            }
        }
        None
    }
}
impl<K: Ord, V> Default for SeqQueue<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V> Len for SeqQueue<K, V> {
    fn len(&self) -> usize {
        self.queue.len()
    }
}
impl<K, V> Clear for SeqQueue<K, V> {
    fn clear(&mut self) {
        if let Some(keys) = &mut self.keys {
            keys.clear();
        }
        self.next = None;
        self.queue.clear();
    }
}

#[derive(Debug, Clone)]
pub struct BTreeSeqQueue<K, V> {
    next: Option<K>,
    queue: BTreeMap<K, V>,
}
impl<K, V> BTreeSeqQueue<K, V> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next: None,
            queue: BTreeMap::new(),
        }
    }
    #[must_use]
    pub fn next(&self) -> Option<&K> {
        self.next.as_ref()
    }
}
impl<K, V> BTreeSeqQueue<K, V>
where
    K: Ord + Clone,
{
    pub fn set_next(&mut self, next: K, mut stale: impl FnMut((K, V))) {
        loop {
            let Some((head, _)) = self.queue.first_key_value() else {
                break;
            };
            if next <= *head {
                break;
            }
            let key = head.clone();
            let value = self.queue.remove(&key).unwrap();
            stale((key, value));
        }
        self.next = Some(next);
    }
}
impl<K, V> BTreeSeqQueue<K, V>
where
    K: Ord + Clone + One + CheckedAdd,
{
    #[must_use]
    pub fn insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) -> SeqInsertResult {
        let case = insert_case(self.next(), &key);
        match case {
            SeqInsertResult::Stalled | SeqInsertResult::InOrder | SeqInsertResult::OutOfOrder => {
                self.force_insert(key, value, &mut waste);
            }
            SeqInsertResult::Stale => {
                waste((key, value));
            }
        }
        case
    }
    #[must_use]
    pub fn insert_pop(
        &mut self,
        key: K,
        value: V,
        mut waste: impl FnMut((K, V)),
    ) -> SeqInsertPopResult<K, V> {
        let case = insert_case(self.next(), &key);
        match case {
            SeqInsertResult::Stalled => {
                self.force_insert(key, value, &mut waste);
                SeqInsertPopResult::Stalled
            }
            SeqInsertResult::Stale => {
                waste((key, value));
                SeqInsertPopResult::Stale
            }
            SeqInsertResult::InOrder => {
                if let Some(ejected) = self.pop() {
                    waste(ejected);
                } else {
                    self.next = key.checked_add(&K::one());
                }
                SeqInsertPopResult::InOrder((key, value))
            }
            SeqInsertResult::OutOfOrder => {
                self.force_insert(key, value, &mut waste);
                SeqInsertPopResult::OutOfOrder
            }
        }
    }
    fn force_insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) {
        if let Some(ejected) = self.queue.insert(key.clone(), value) {
            waste((key, ejected));
        }
    }
    #[must_use]
    pub fn peak(&self) -> Option<(&K, &V)> {
        let key = self.next()?;
        Some((key, self.queue.get(key)?))
    }
    #[must_use]
    pub fn pop(&mut self) -> Option<(K, V)> {
        let key = self.next()?.clone();
        let value = self.queue.remove(&key)?;
        self.next = key.checked_add(&K::one());
        Some((key, value))
    }
    pub fn insert_pop_all<O>(
        &mut self,
        key: K,
        value: V,
        waste: impl FnMut((K, V)),
        mut read: impl FnMut((K, V)) -> ControlFlow<O>,
    ) -> Option<O> {
        let (k, v) = self.insert_pop(key, value, waste).into_in_order()?;
        let ctrl = read((k, v));
        if let ControlFlow::Break(o) = ctrl {
            return Some(o);
        }
        while let Some((k, v)) = self.pop() {
            let ctrl = read((k, v));
            if let ControlFlow::Break(o) = ctrl {
                return Some(o);
            }
        }
        None
    }
}
impl<K, V> Default for BTreeSeqQueue<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V> Len for BTreeSeqQueue<K, V> {
    fn len(&self) -> usize {
        self.queue.len()
    }
}
impl<K, V> Clear for BTreeSeqQueue<K, V> {
    fn clear(&mut self) {
        self.next = None;
        self.queue.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqInsertResult {
    Stalled,
    Stale,
    InOrder,
    OutOfOrder,
}
#[must_use]
fn insert_case<K>(next: Option<&K>, key: &K) -> SeqInsertResult
where
    K: Ord,
{
    let Some(next) = next else {
        return SeqInsertResult::Stalled;
    };
    if *key < *next {
        return SeqInsertResult::Stale;
    }
    if *key == *next {
        return SeqInsertResult::InOrder;
    }
    SeqInsertResult::OutOfOrder
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqInsertPopResult<K, V> {
    Stalled,
    Stale,
    InOrder((K, V)),
    OutOfOrder,
}
impl<K, V> SeqInsertPopResult<K, V> {
    pub fn into_in_order(self) -> Option<(K, V)> {
        match self {
            SeqInsertPopResult::InOrder((k, v)) => Some((k, v)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_queue() {
        let q = [SeqQueue::new(), SeqQueue::new_unstable()];
        for mut q in q {
            assert!(q.insert_pop(1, 1, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(2, 2, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(3, 3, |_| {}).into_in_order().is_none());
            assert_eq!(q.len(), 3);
            q.set_next(2, |(k, v)| {
                assert_eq!(k, v);
                assert_eq!(k, 1);
            });
            assert_eq!(q.len(), 2);
            assert!(q.insert_pop(1, 1, |_| {}).into_in_order().is_none());
            assert_eq!(q.len(), 2);
            assert_eq!(q.insert_pop(2, 2, |_| {}).into_in_order().unwrap(), (2, 2));
            assert_eq!(q.len(), 1);
            assert_eq!(q.pop(|_| {}).unwrap(), (3, 3));
            assert!(q.insert_pop(6, 6, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(5, 5, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(5, 5, |_| {}).into_in_order().is_none());
            let mut start = 4;
            let res: Option<()> = q.insert_pop_all(
                4,
                4,
                |_| {},
                |(k, v)| {
                    assert_eq!(k, v);
                    assert_eq!(start, k);
                    start += 1;
                    ControlFlow::Continue(())
                },
            );
            dbg!(&q);
            assert!(res.is_none());
            assert_eq!(start, 7);
            assert!(q.pop(|_| {}).is_none());
        }
    }
    #[test]
    fn test_b_tree_seq_queue() {
        let q = [BTreeSeqQueue::new()];
        for mut q in q {
            assert!(q.insert_pop(1, 1, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(2, 2, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(3, 3, |_| {}).into_in_order().is_none());
            assert_eq!(q.len(), 3);
            q.set_next(2, |(k, v)| {
                assert_eq!(k, v);
                assert_eq!(k, 1);
            });
            assert_eq!(q.len(), 2);
            assert!(q.insert_pop(1, 1, |_| {}).into_in_order().is_none());
            assert_eq!(q.len(), 2);
            assert_eq!(q.insert_pop(2, 2, |_| {}).into_in_order().unwrap(), (2, 2));
            assert_eq!(q.len(), 1);
            assert_eq!(q.pop().unwrap(), (3, 3));
            assert!(q.insert_pop(6, 6, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(5, 5, |_| {}).into_in_order().is_none());
            assert!(q.insert_pop(5, 5, |_| {}).into_in_order().is_none());
            let mut start = 4;
            let res: Option<()> = q.insert_pop_all(
                4,
                4,
                |_| {},
                |(k, v)| {
                    assert_eq!(k, v);
                    assert_eq!(start, k);
                    start += 1;
                    ControlFlow::Continue(())
                },
            );
            dbg!(&q);
            assert!(res.is_none());
            assert_eq!(start, 7);
            assert!(q.pop().is_none());
        }
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use test::Bencher;

    use super::*;

    const SEG_LEN: usize = 1 << 7;
    const N: usize = 1 << 14;

    macro_rules! insert_pop {
        ($bencher: ident, $q: ident) => {
            $bencher.iter(|| {
                $q.set_next(0, |_| {});
                let mut rev = false;
                for round in 0..(N / SEG_LEN) {
                    let start = round * SEG_LEN;
                    assert_eq!(*$q.next().unwrap(), start);
                    for i in 0..SEG_LEN {
                        let i = if rev {
                            start + (SEG_LEN - 1 - i)
                        } else {
                            start + i
                        };
                        $q.insert_pop_all(i, i, |_| {}, |_| ControlFlow::<()>::Continue(()));
                    }
                    rev = !rev;
                }
            });
        };
    }

    #[bench]
    fn bench_unstable_seq_queue(bencher: &mut Bencher) {
        let mut q = SeqQueue::new_unstable();
        insert_pop!(bencher, q);
    }
    #[bench]
    fn bench_seq_queue(bencher: &mut Bencher) {
        let mut q = SeqQueue::new();
        insert_pop!(bencher, q);
    }
    #[bench]
    fn bench_b_tree(bencher: &mut Bencher) {
        let mut q = BTreeSeqQueue::new();
        insert_pop!(bencher, q);
    }
}
