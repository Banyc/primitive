use core::{hash::Hash, num::NonZeroUsize, ops::ControlFlow};
use std::collections::{BTreeMap, HashSet};

use num_traits::{CheckedAdd, CheckedSub, NumCast, One};

use crate::{
    map::MapInsert,
    ops::len::{Capacity, Full, Len},
    queue::ordered_queue::OrderedQueue,
    Clear,
};

use super::fixed_queue::BitQueue;

#[derive(Debug, Clone)]
pub struct SeqQueue<K, V> {
    queue: OrderedQueue<K, V>,
    next: Option<K>,
    /// There could be `K` in [`Self::queue`] that is not covered by [`Self::keys`]
    keys: Option<SeqQueueKeys<K>>,
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
    #[must_use]
    pub fn new(window_size_at_least: NonZeroUsize) -> Self {
        let mut win = BitQueue::new(window_size_at_least.get());
        reset_bit_win(&mut win);
        Self {
            queue: OrderedQueue::new(),
            next: None,
            keys: Some(SeqQueueKeys {
                win,
                sparse: HashSet::new(),
            }),
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
    K: Ord + CheckedSub + NumCast + Hash,
{
    pub fn set_next(&mut self, next: K, mut stale: impl FnMut((K, V))) {
        loop {
            let Some((head, _)) = self.queue.peek() else {
                break;
            };
            if next <= *head {
                break;
            }
            if let Some(SeqQueueKeys { win: _, sparse }) = &mut self.keys {
                assert!(sparse.remove(head));
            }
            stale(self.queue.pop().unwrap());
        }
        if let Some(SeqQueueKeys { win, sparse }) = &mut self.keys {
            reset_bit_win(win);
            for key in sparse.iter() {
                let Some(index) = key_index(&next, key) else {
                    continue;
                };
                win.set(index, true);
            }
            sparse.clear();
        }
        self.next = Some(next);
    }
}
impl<K, V> SeqQueue<K, V>
where
    K: Ord + CheckedAdd + One + Clone + CheckedSub + NumCast + Hash,
{
    #[must_use]
    pub fn peek(&self) -> Option<(&K, &V)> {
        let next = self.next()?;
        let (k, v) = self.queue.peek()?;
        if k != next {
            return None;
        }
        Some((k, v))
    }
    #[must_use]
    pub fn pop(&mut self, waste: impl FnMut((K, V))) -> Option<(K, V)> {
        let _ = self.peek()?;
        let (k, v) = self.queue.pop().unwrap();
        if let Some(SeqQueueKeys { win, sparse: _ }) = &mut self.keys {
            win.dequeue().unwrap();
            win.enqueue(false);
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
        while let Some((k, _)) = self.queue.peek() {
            if k != next {
                break;
            }
            waste(self.queue.pop().unwrap());
        }
    }
    #[must_use]
    pub fn insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) -> SeqInsertResult {
        let win_size = self.keys.as_ref().map(|keys| keys.win.capacity());
        let case = insert_case(self.next(), &key, win_size);
        match case {
            SeqInsertResult::Stalled | SeqInsertResult::InOrder | SeqInsertResult::OutOfOrder => {
                self.force_insert(key, value, &mut waste);
            }
            SeqInsertResult::Stale | SeqInsertResult::OutOfWindow => {
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
        let win_size = self.keys.as_ref().map(|keys| keys.win.capacity());
        let case = insert_case(self.next(), &key, win_size);
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
            SeqInsertResult::OutOfWindow => {
                waste((key, value));
                SeqInsertPopResult::OutOfWindow
            }
        }
    }
    fn force_insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) {
        if let Some(SeqQueueKeys { win, sparse }) = &mut self.keys {
            let mut is_duped = || {
                match &self.next {
                    Some(next) => {
                        let Some(index) = key_index(next, &key) else {
                            return true;
                        };
                        if win.get(index) {
                            return true;
                        }
                        win.set(index, true);
                    }
                    None => {
                        if sparse.contains(&key) {
                            return true;
                        }
                        sparse.insert(key.clone());
                    }
                }
                false
            };
            if is_duped() {
                waste((key, value));
                return;
            }
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
impl<K, V> Len for SeqQueue<K, V> {
    fn len(&self) -> usize {
        self.queue.len()
    }
}
impl<K, V> Clear for SeqQueue<K, V> {
    fn clear(&mut self) {
        if let Some(SeqQueueKeys { win, sparse }) = &mut self.keys {
            reset_bit_win(win);
            sparse.clear();
        }
        self.next = None;
        self.queue.clear();
    }
}
#[derive(Debug, Clone)]
struct SeqQueueKeys<K> {
    pub win: BitQueue,
    pub sparse: HashSet<K>,
}
fn key_index<K>(next: &K, key: &K) -> Option<usize>
where
    K: CheckedSub + NumCast,
{
    let index = key.checked_sub(next)?.to_usize()?;
    Some(index)
}
fn reset_bit_win(win: &mut BitQueue) {
    win.clear();
    for _ in 0..win.capacity() {
        win.enqueue(false);
    }
    assert!(win.is_full());
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
    K: Ord + Clone + One + CheckedAdd + CheckedSub + NumCast,
{
    #[must_use]
    pub fn insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) -> SeqInsertResult {
        let case = insert_case(self.next(), &key, None);
        match case {
            SeqInsertResult::Stalled | SeqInsertResult::InOrder | SeqInsertResult::OutOfOrder => {
                self.force_insert(key, value, &mut waste);
            }
            SeqInsertResult::Stale => {
                waste((key, value));
            }
            SeqInsertResult::OutOfWindow => panic!(),
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
        let case = insert_case(self.next(), &key, None);
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
            SeqInsertResult::OutOfWindow => panic!(),
        }
    }
    fn force_insert(&mut self, key: K, value: V, mut waste: impl FnMut((K, V))) {
        if let Some(ejected) = self.queue.insert(key.clone(), value) {
            waste((key, ejected));
        }
    }
    #[must_use]
    pub fn peek(&self) -> Option<(&K, &V)> {
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
    OutOfWindow,
}
#[must_use]
fn insert_case<K>(next: Option<&K>, key: &K, win_size: Option<usize>) -> SeqInsertResult
where
    K: Ord + CheckedSub + NumCast,
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
    if let Some(win_size) = win_size {
        let Some(diff) = key.checked_sub(next).unwrap().to_usize() else {
            return SeqInsertResult::OutOfWindow;
        };
        if win_size <= diff {
            return SeqInsertResult::OutOfWindow;
        }
    }
    SeqInsertResult::OutOfOrder
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqInsertPopResult<K, V> {
    Stalled,
    Stale,
    InOrder((K, V)),
    OutOfOrder,
    OutOfWindow,
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
        let q = [
            SeqQueue::new(NonZeroUsize::new(1 << 10).unwrap()),
            SeqQueue::new_unstable(),
        ];
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
        let mut q = SeqQueue::new(NonZeroUsize::new(1 << 10).unwrap());
        insert_pop!(bencher, q);
    }
    #[bench]
    fn bench_b_tree(bencher: &mut Bencher) {
        let mut q = BTreeSeqQueue::new();
        insert_pop!(bencher, q);
    }
}
