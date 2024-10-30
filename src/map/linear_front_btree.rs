use core::borrow::Borrow;
use std::collections::BTreeMap;

use crate::{
    arena::stack::{Stack, StaticStack},
    seq::{LinearSearch, Seq, SeqMut},
    Full, Len, LenExt,
};

pub type LinearFrontBTreeMap11<K, V> = LinearFrontBTreeMap<K, V, 11>;

const REFILL_RATIO: f64 = 5. / 9.;

/// It is optimal if:
///
/// - insertions and removals are occasional
/// - searching takes most of the time
/// - value size is small
///
/// Linear size `N` is restricted by:
///
/// - frequency of insertions and removals
/// - value size
#[derive(Debug, Clone)]
pub struct LinearFrontBTreeMap<K, V, const N: usize> {
    btree_first: Option<K>,
    linear: StaticStack<OrdEntry<K, V>, N>,
    btree: BTreeMap<K, V>,
}
impl<K, V, const N: usize> LinearFrontBTreeMap<K, V, N>
where
    K: Ord + Clone,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.btree_first.is_some() && *self.btree_first.as_ref().unwrap() <= key {
            return self.btree.insert(key, value);
        }
        let linear_last = self.linear.as_slice().last();
        if self.linear.is_full() && (linear_last.is_none() || linear_last.unwrap().key < key) {
            if self.btree_first.is_none() || key < *self.btree_first.as_ref().unwrap() {
                self.btree_first = Some(key.clone());
            }
            return self.btree.insert(key, value);
        }
        let linear_insert_index = match self.linear.linear_search_by(|entry| entry.key.cmp(&key)) {
            Ok(i) => {
                let old = core::mem::replace(&mut self.linear.as_slice_mut()[i].value, value);
                return Some(old);
            }
            Err(i) => i,
        };
        let last = self
            .linear
            .insert(linear_insert_index, OrdEntry { key, value });
        if let Some(last) = last {
            self.btree_first = Some(last.key.clone());
            assert!(self.btree.insert(last.key, last.value).is_none());
        }
        None
    }
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q>,
    {
        for entry in self.linear.as_slice() {
            if entry.key.borrow() == key {
                return Some(&entry.value);
            }
        }
        self.btree.get(key)
    }
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q>,
    {
        for entry in self.linear.as_slice_mut() {
            if entry.key.borrow() == key {
                return Some(&mut entry.value);
            }
        }
        self.btree.get_mut(key)
    }
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Ord + ?Sized,
        K: Borrow<Q>,
    {
        if let Some(btree_first) = &self.btree_first {
            if btree_first.borrow() <= key {
                let removed = self.btree.remove(key);
                if btree_first.borrow() == key {
                    self.reset_btree_first();
                }
                return removed;
            }
        }
        let index = self
            .linear
            .linear_search_by(|entry| {
                let entry_key: &Q = entry.key.borrow();
                entry_key.cmp(key)
            })
            .ok()?;
        let removed = self.linear.remove(index).value;
        self.refill_linear();
        Some(removed)
    }
    #[must_use]
    pub fn pop_first(&mut self) -> Option<(K, V)> {
        if !self.linear.is_empty() {
            let entry = self.linear.remove(0);
            self.refill_linear();
            return Some((entry.key, entry.value));
        }
        let first = self.btree.pop_first();
        self.reset_btree_first();
        first
    }
    #[must_use]
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        if let Some(last) = self.btree.pop_last() {
            if self.btree.is_empty() {
                self.btree_first = None;
            }
            return Some(last);
        }
        self.linear.pop().map(|entry| (entry.key, entry.value))
    }
    fn refill_linear(&mut self) {
        let refill_len = (N as f64 * REFILL_RATIO) as usize;
        if refill_len < self.linear.len() {
            return;
        }
        if self.btree_first.is_none() {
            return;
        }
        loop {
            if self.linear.is_full() {
                break;
            }
            let Some((last_key, last_value)) = self.btree.pop_first() else {
                break;
            };
            self.linear.push(OrdEntry {
                key: last_key,
                value: last_value,
            });
        }
        self.reset_btree_first();
    }
    fn reset_btree_first(&mut self) {
        self.btree_first = self.btree.first_key_value().map(|(key, _)| key.clone());
    }
}
impl<K, V, const N: usize> LinearFrontBTreeMap<K, V, N> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            btree_first: None,
            linear: StaticStack::new(),
            btree: BTreeMap::new(),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
        self.linear
            .as_slice()
            .iter()
            .map(|entry| (&entry.key, &entry.value))
            .chain(self.btree.iter())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> + '_ {
        self.linear
            .as_slice_mut()
            .iter_mut()
            .map(|entry| (&entry.key, &mut entry.value))
            .chain(self.btree.iter_mut())
    }
}
impl<K, V, const N: usize> Len for LinearFrontBTreeMap<K, V, N> {
    fn len(&self) -> usize {
        self.linear.len() + self.btree.len()
    }
}
impl<K, V, const N: usize> Default for LinearFrontBTreeMap<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
struct OrdEntry<K, V> {
    pub key: K,
    pub value: V,
}
impl<K, V> PartialEq for OrdEntry<K, V>
where
    K: Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<K, V> Eq for OrdEntry<K, V> where K: Eq {}
impl<K, V> Ord for OrdEntry<K, V>
where
    K: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}
impl<K, V> PartialOrd for OrdEntry<K, V>
where
    K: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_front_btree() {
        let end = 21;
        let mut tree = LinearFrontBTreeMap11::new();
        for i in 0..end {
            assert_eq!(tree.len(), i);
            tree.insert(i, i);
            assert_eq!(*tree.get(&0).unwrap(), 0);
            assert_eq!(*tree.get(&i).unwrap(), i);
        }
        for i in 0..end {
            assert_eq!(tree.len(), end - i);
            assert_eq!(*tree.get(&(end - 1)).unwrap(), end - 1);
            tree.remove(&i);
            assert!(tree.get(&i).is_none());
        }
        assert!(tree.is_empty());
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use std::hint::black_box;

    use test::Bencher;

    use crate::sync::tests::RepeatedData;

    use super::*;
    const LINEAR: usize = 11;
    const DATA_SIZE: usize = (1 << 4) + (1 << 5);

    #[bench]
    fn bench_insert_remove_linear_front_btree(bencher: &mut Bencher) {
        let mut b: LinearFrontBTreeMap<usize, RepeatedData<u8, DATA_SIZE>, LINEAR> =
            LinearFrontBTreeMap::new();
        bencher.iter(|| {
            for i in (0..(LINEAR * 2)).rev() {
                b.insert(i, RepeatedData::new(i as _));
            }
            for i in 0..(LINEAR * 2) {
                b.remove(&i);
            }
        });
    }
    #[bench]
    fn bench_insert_remove_btree(bencher: &mut Bencher) {
        let mut b: BTreeMap<usize, RepeatedData<u8, DATA_SIZE>> = BTreeMap::new();
        bencher.iter(|| {
            for i in (0..(LINEAR * 2)).rev() {
                b.insert(i, RepeatedData::new(i as _));
            }
            for i in 0..(LINEAR * 2) {
                b.remove(&i);
            }
        });
    }

    #[bench]
    fn bench_iter_linear_front_btree(bencher: &mut Bencher) {
        let mut b: LinearFrontBTreeMap<usize, RepeatedData<u8, DATA_SIZE>, LINEAR> =
            LinearFrontBTreeMap::new();
        for i in 0..(LINEAR * 2) {
            b.insert(i, RepeatedData::new(i as _));
        }
        assert_eq!(b.linear.len(), LINEAR);
        bencher.iter(|| {
            for (k, v) in b.iter() {
                black_box(k);
                black_box(v);
            }
        });
    }
    #[bench]
    fn bench_iter_btree(bencher: &mut Bencher) {
        let mut b: BTreeMap<usize, RepeatedData<u8, DATA_SIZE>> = BTreeMap::new();
        for i in 0..(LINEAR * 2) {
            b.insert(i, RepeatedData::new(i as _));
        }
        bencher.iter(|| {
            for (k, v) in b.iter() {
                black_box(k);
                black_box(v);
            }
        });
    }
}
