use std::{borrow::Borrow, collections::BTreeMap};

use crate::{
    arena::stack::{Stack, StaticStack},
    seq::{Seq, SeqMut},
    Capacity, Len, LenExt,
};

pub type LinearFrontBTreeMap20<K, V> = LinearFrontBTreeMap<K, V, 20>;

#[derive(Debug)]
pub struct LinearFrontBTreeMap<K, V, const N: usize> {
    linear: StaticStack<OrdEntry<K, V>, N>,
    btree: BTreeMap<K, V>,
}
impl<K, V, const N: usize> LinearFrontBTreeMap<K, V, N>
where
    K: Ord,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut linear_insert_index = None;
        for (i, entry) in self.linear.as_slice_mut().iter_mut().enumerate() {
            if entry.key < key {
                continue;
            }
            if entry.key == key {
                let old = core::mem::replace(&mut entry.value, value);
                return Some(old);
            }
            linear_insert_index = Some(i);
            break;
        }
        if linear_insert_index.is_none() && self.linear.len() != self.linear.capacity() {
            self.linear.push(OrdEntry { key, value });
            return None;
        }
        if let Some(index) = linear_insert_index {
            let last = self.linear.insert(index, OrdEntry { key, value });
            if let Some(last) = last {
                assert!(self.btree.insert(last.key, last.value).is_none());
            }
            return None;
        }
        self.btree.insert(key, value)
    }
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
        let mut linear_index = None;
        for (i, entry) in self.linear.as_slice().iter().enumerate() {
            if entry.key.borrow() == key {
                linear_index = Some(i);
                break;
            }
        }
        if let Some(index) = linear_index {
            let removed = self.linear.remove(index).value;
            if let Some((key, value)) = self.btree.pop_first() {
                self.linear.push(OrdEntry { key, value });
            }
            return Some(removed);
        }
        self.btree.remove(key)
    }
    pub fn pop_first(&mut self) -> Option<(K, V)> {
        if !self.linear.is_empty() {
            let entry = self.linear.remove(0);
            if let Some((key, value)) = self.btree.pop_first() {
                self.linear.push(OrdEntry { key, value });
            }
            return Some((entry.key, entry.value));
        }
        self.btree.pop_first()
    }
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        if let Some(last) = self.btree.pop_last() {
            return Some(last);
        }
        self.linear.pop().map(|entry| (entry.key, entry.value))
    }
}
impl<K, V, const N: usize> LinearFrontBTreeMap<K, V, N> {
    pub fn new() -> Self {
        Self {
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
impl<K: Clone + Copy, V: Clone + Copy, const N: usize> Clone for LinearFrontBTreeMap<K, V, N> {
    fn clone(&self) -> Self {
        Self {
            linear: self.linear,
            btree: self.btree.clone(),
        }
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
        let mut tree = LinearFrontBTreeMap20::new();
        for i in 0..21 {
            tree.insert(i, i);
            assert_eq!(*tree.get(&i).unwrap(), i);
        }
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use std::hint::black_box;

    use test::Bencher;

    use crate::sync::tests::RepeatedData;

    use super::*;
    const LINEAR: usize = 20;
    const DATA_SIZE: usize = 1 << 4;

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
