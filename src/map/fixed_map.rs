use core::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
    num::NonZeroUsize,
};
use std::hash::RandomState;

use crate::ops::ring::RingSpace;

#[derive(Debug, Clone)]
pub struct FixedHashMap<K, V, H = RandomState> {
    entries: Vec<Option<(K, V)>>,
    direct_sets: NonZeroUsize,
    assoc_ways: NonZeroUsize,
    next_way_index: usize,
    hash_builder: H,
}
impl<K, V, H> FixedHashMap<K, V, H> {
    #[must_use]
    pub fn with_hasher(direct_sets: NonZeroUsize, assoc_ways: NonZeroUsize, hasher: H) -> Self {
        Self {
            entries: (0..direct_sets.get() * assoc_ways.get())
                .map(|_| None)
                .collect(),
            direct_sets,
            assoc_ways,
            next_way_index: 0,
            hash_builder: hasher,
        }
    }
}
impl<K, V> FixedHashMap<K, V, RandomState> {
    #[must_use]
    pub fn new(direct_sets: NonZeroUsize, assoc_ways: NonZeroUsize) -> Self {
        Self::with_hasher(direct_sets, assoc_ways, RandomState::new())
    }
}
impl<K, V, H> FixedHashMap<K, V, H>
where
    K: Eq + Hash,
    H: BuildHasher,
{
    pub fn insert(&mut self, key: K, mut value: impl FnMut(usize) -> V) -> (usize, Option<(K, V)>) {
        let hash = self.hash_builder.hash_one(&key);
        if let Some(index) = self.get_index_pre_hashed(&key, hash) {
            let old = self.entries[index].take().unwrap();
            self.entries[index] = Some((key, value(index)));
            return (index, Some(old));
        }
        let set_index = self.set_index(hash);
        let ways = &self.entries[self.ways(set_index)];
        let way_index = ways.iter().position(|entry| entry.is_none());
        let way_index = way_index.unwrap_or(self.next_way_index);
        if self.assoc_ways.get() != 1 {
            self.next_way_index = self.next_way_index.ring_add(1, self.assoc_ways.get() - 1);
        }
        let index = self.index(set_index, way_index);
        let ejected = match &mut self.entries[index] {
            Some((k, v)) => {
                let k = core::mem::replace(k, key);
                let v = core::mem::replace(v, value(index));
                Some((k, v))
            }
            None => {
                self.entries[index] = Some((key, value(index)));
                None
            }
        };
        (index, ejected)
    }
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        let index = self.get_index(key)?;
        self.entries[index].take().map(|(_, v)| v)
    }
    pub fn remove_entry(&mut self, index: usize) -> Option<(K, V)> {
        self.entries[index].take()
    }
    #[must_use]
    pub fn entry(&self, index: usize) -> Option<(&K, &V)> {
        let (k, v) = self.entries[index].as_ref()?;
        Some((k, v))
    }
    #[must_use]
    pub fn entry_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        let (k, v) = self.entries[index].as_mut()?;
        Some((k, v))
    }
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        let index = self.get_index(key)?;
        let (_, v) = self.entries[index].as_ref()?;
        Some(v)
    }
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        let index = self.get_index(key)?;
        let (_, v) = self.entries[index].as_mut()?;
        Some(v)
    }
    #[must_use]
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        Q: Eq + Hash + ?Sized,
        K: Borrow<Q>,
    {
        let hash = self.hash_builder.hash_one(key);
        self.get_index_pre_hashed(key, hash)
    }
    #[must_use]
    fn get_index_pre_hashed<Q>(&self, key: &Q, hash: u64) -> Option<usize>
    where
        Q: Eq + ?Sized,
        K: Borrow<Q>,
    {
        let set_index = self.set_index(hash);
        let ways = &self.entries[self.ways(set_index)];
        let predicate = |entry: &Option<(K, V)>| {
            let Some((k, _)) = entry else {
                return false;
            };
            k.borrow() == key
        };
        let way_index = if ways.len() == 1 {
            if predicate(&ways[0]) {
                Some(0)
            } else {
                None
            }
        } else {
            ways.iter().position(predicate)
        };
        let index = self.index(set_index, way_index?);
        Some(index)
    }
    #[must_use]
    fn index(&self, set_index: usize, way_index: usize) -> usize {
        set_index * self.assoc_ways.get() + way_index
    }
    #[must_use]
    fn ways(&self, set_index: usize) -> core::ops::Range<usize> {
        let start = set_index * self.assoc_ways.get();
        let end = start + self.assoc_ways.get();
        start..end
    }
    #[must_use]
    fn set_index(&self, hash: u64) -> usize {
        hash as usize % self.direct_sets.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_map() {
        let direct_sets = NonZeroUsize::new(4).unwrap();
        let assoc_ways = NonZeroUsize::new(1).unwrap();
        let mut map = FixedHashMap::new(direct_sets, assoc_ways);
        map.insert(1, |_| 1);
        assert_eq!(*map.get_mut(&1).unwrap(), 1);
        map.insert(2, |_| 2);
        assert_eq!(*map.get_mut(&2).unwrap(), 2);
        map.insert(3, |_| 3);
        assert_eq!(*map.get_mut(&3).unwrap(), 3);
        map.insert(4, |_| 4);
        assert_eq!(*map.get_mut(&4).unwrap(), 4);
        map.insert(5, |_| 5);
        assert_eq!(*map.get_mut(&5).unwrap(), 5);
        dbg!(&map);

        let direct_sets = NonZeroUsize::new(5).unwrap();
        let assoc_ways = NonZeroUsize::new(2).unwrap();
        let mut map = FixedHashMap::new(direct_sets, assoc_ways);
        map.insert(1, |_| 1);
        assert_eq!(*map.get_mut(&1).unwrap(), 1);
        map.insert(2, |_| 2);
        assert_eq!(*map.get_mut(&2).unwrap(), 2);
        map.insert(3, |_| 3);
        assert_eq!(*map.get_mut(&3).unwrap(), 3);
        map.insert(4, |_| 4);
        assert_eq!(*map.get_mut(&4).unwrap(), 4);
        map.insert(5, |_| 5);
        assert_eq!(*map.get_mut(&5).unwrap(), 5);
        map.insert(6, |_| 6);
        assert_eq!(*map.get_mut(&6).unwrap(), 6);
    }

    #[test]
    #[ignore]
    fn test_load_factors() {
        const SAMPLES: usize = 1 << 10;
        let load_factors = [
            0.01, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95, 1.,
        ];
        let buildup_hasher = RandomState::new();
        let saturated_hasher = RandomState::new();
        for load_factor in load_factors {
            let len = SAMPLES + ((1. / load_factor - 1.) * SAMPLES as f64) as usize;
            let mut space = vec![0; len];
            let mut buildup_collisions = 0;
            for sample in 0..SAMPLES {
                let hash = buildup_hasher.hash_one(sample);
                let i = hash as usize % space.len();
                if 0 < space[i] {
                    buildup_collisions += 1;
                }
                space[i] += 1;
            }
            let buildup_rate = buildup_collisions as f64 / SAMPLES as f64;
            let mut saturated_collisions = 0;
            for sample in 0..SAMPLES {
                let hash = saturated_hasher.hash_one(sample);
                let i = hash as usize % space.len();
                if 0 < space[i] {
                    saturated_collisions += 1;
                }
            }
            let max_hits = space.iter().max().unwrap();
            let mean_hits = space
                .iter()
                .map(|&x| x as f64 / space.len() as f64)
                .sum::<f64>();
            let saturated_rate = saturated_collisions as f64 / SAMPLES as f64;
            let len_rate = len as f64 / SAMPLES as f64;
            println!("load factor: {load_factor}; buildup rate: {buildup_rate}; saturated rate: {saturated_rate}; len rate: {len_rate}; max: {max_hits}; mean: {mean_hits};");
        }
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use std::collections::HashMap;

    use test::Bencher;

    use crate::sync::tests::RepeatedData;

    use super::*;

    const DIRECT_SETS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1 << 10) };
    const ASSOC_WAYS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1 << 2) };
    // const ASSOC_WAYS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
    const DATA_SIZE: usize = 1 << 6;
    const N: usize = 1 << 9;

    #[bench]
    fn bench_fixed_map(bencher: &mut Bencher) {
        let mut map: FixedHashMap<usize, RepeatedData<u8, DATA_SIZE>> =
            FixedHashMap::new(DIRECT_SETS, ASSOC_WAYS);
        bencher.iter(|| {
            for i in 0..N {
                map.insert(i, |_| RepeatedData::new(i as _));
            }
            for i in 0..N {
                map.remove(&i);
            }
        });
    }
    #[bench]
    fn bench_hash_map(bencher: &mut Bencher) {
        let mut map: HashMap<usize, RepeatedData<u8, DATA_SIZE>> = HashMap::new();
        bencher.iter(|| {
            for i in 0..N {
                map.insert(i, RepeatedData::new(i as _));
            }
            for i in 0..N {
                map.remove(&i);
            }
        });
    }
}
