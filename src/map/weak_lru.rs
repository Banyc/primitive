use std::{
    borrow::Borrow,
    hash::{BuildHasher, RandomState},
    num::NonZeroUsize,
};

use crate::ops::ring::RingSpace;

use super::fixed_map::FixedHashMap;

#[derive(Debug, Clone)]
pub struct WeakLru<K, V, const N: usize, H = RandomState> {
    keys: FixedHashMap<K, usize, H>,
    next_evict: usize,
    values: [Option<Entry<V>>; N],
}
impl<K, V, const N: usize> WeakLru<K, V, N, RandomState> {
    #[must_use]
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }
}
impl<K, V, const N: usize, H> WeakLru<K, V, N, H> {
    const EVICT_WINDOW: usize = 4;
    const LOAD_FACTOR: f64 = 0.75;
    #[must_use]
    pub fn with_hasher(hasher: H) -> Self {
        assert!(Self::EVICT_WINDOW <= N);
        let hash_map_size =
            NonZeroUsize::new(N + (N as f64 * (1. / Self::LOAD_FACTOR - 1.)) as usize).unwrap();
        let values = (0..N)
            .map(|_| None)
            .collect::<Vec<_>>()
            .try_into()
            .ok()
            .unwrap();
        Self {
            keys: FixedHashMap::with_hasher(hash_map_size, hasher),
            values,
            next_evict: 0,
        }
    }
}
impl<K, V, const N: usize> Default for WeakLru<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K, V, const N: usize, H> WeakLru<K, V, N, H>
where
    K: Eq + core::hash::Hash,
    H: BuildHasher,
{
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + core::hash::Hash,
    {
        let index = *self.keys.get(key)?;
        Some(self.values[index].as_mut().unwrap().access())
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(&index) = self.keys.get(&key) {
            *self.values[index].as_mut().unwrap().access() = value;
            return;
        }
        let mut least_access_times: Option<usize> = None;
        let mut value_index: Option<usize> = None;
        for i in 0..Self::EVICT_WINDOW {
            let i = self.next_evict.ring_add(i, self.values.len() - 1);
            let Some(entry) = &self.values[i] else {
                value_index = Some(i);
                continue;
            };
            if least_access_times.is_none() || entry.times() < least_access_times.unwrap() {
                least_access_times = Some(entry.times());
                value_index = Some(i);
            }
            self.values[i].as_mut().unwrap().reset_times();
        }
        if Self::EVICT_WINDOW < self.values.len() {
            self.next_evict = self
                .next_evict
                .ring_add(Self::EVICT_WINDOW, self.values.len() - 1);
        }
        let value_index = value_index.unwrap();
        let entry = self.values[value_index].take();
        if let Some(entry) = entry {
            self.keys.remove_entry(entry.key_index);
        }
        let (key_index, collided) = self.keys.insert(key, |_| value_index);
        if let Some((_, value_index)) = collided {
            self.values[value_index] = None;
        }
        self.values[value_index] = Some(Entry::new(value, key_index));
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry<V> {
    value: V,
    key_index: usize,
    times: usize,
}
impl<V> Entry<V> {
    #[must_use]
    pub fn new(value: V, key_index: usize) -> Self {
        Self {
            value,
            key_index,
            times: 1,
        }
    }
    pub fn times(&self) -> usize {
        self.times
    }
    pub fn reset_times(&mut self) {
        self.times = 0;
    }
    pub fn access(&mut self) -> &mut V {
        self.times = self.times.saturating_add(1);
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_lru() {
        let mut lru: WeakLru<_, _, 4> = WeakLru::new();
        lru.insert(1, 1);
        assert_eq!(*lru.get_mut(&1).unwrap(), 1);
        lru.insert(2, 2);
        assert_eq!(*lru.get_mut(&2).unwrap(), 2);
        lru.insert(3, 3);
        assert_eq!(*lru.get_mut(&3).unwrap(), 3);
        lru.insert(4, 4);
        assert_eq!(*lru.get_mut(&4).unwrap(), 4);
        lru.insert(5, 5);
        assert_eq!(*lru.get_mut(&5).unwrap(), 5);
        dbg!(&lru);

        let mut lru: WeakLru<_, _, 5> = WeakLru::new();
        lru.insert(1, 1);
        assert_eq!(*lru.get_mut(&1).unwrap(), 1);
        lru.insert(2, 2);
        assert_eq!(*lru.get_mut(&2).unwrap(), 2);
        lru.insert(3, 3);
        assert_eq!(*lru.get_mut(&3).unwrap(), 3);
        lru.insert(4, 4);
        assert_eq!(*lru.get_mut(&4).unwrap(), 4);
        lru.insert(5, 5);
        assert_eq!(*lru.get_mut(&5).unwrap(), 5);
        lru.insert(6, 6);
        assert_eq!(*lru.get_mut(&6).unwrap(), 6);
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use test::Bencher;

    use crate::sync::tests::RepeatedData;

    use super::*;

    const LRU_SIZE: usize = 1 << 8;
    const DATA_SIZE: usize = 1 << 6;
    const N: usize = 1 << 10;

    #[bench]
    fn bench_weak_lru(bencher: &mut Bencher) {
        let mut lru: WeakLru<usize, RepeatedData<u8, DATA_SIZE>, LRU_SIZE, lru::DefaultHasher> =
            WeakLru::with_hasher(lru::DefaultHasher::default());
        bencher.iter(|| {
            for i in 0..N {
                lru.insert(i, RepeatedData::new(i as _));
            }
        });
    }
    #[bench]
    fn bench_lru(bencher: &mut Bencher) {
        let mut lru: lru::LruCache<usize, RepeatedData<u8, DATA_SIZE>> =
            lru::LruCache::new(NonZeroUsize::new(LRU_SIZE).unwrap());
        bencher.iter(|| {
            for i in 0..N {
                lru.put(i, RepeatedData::new(i as _));
            }
        });
    }
}
