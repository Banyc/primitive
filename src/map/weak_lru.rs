use std::{
    borrow::Borrow,
    hash::{BuildHasher, RandomState},
    num::NonZeroUsize,
};

use crate::ops::ring::RingSpace;

use super::fixed_map::{FixedHashMap, GetOrInsert};

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
    /// 2% wrongful key eviction rate
    const KEYS_LOAD_FACTOR: f64 = 0.2;
    const KEYS_ASSOC_WAYS: usize = 2;
    #[must_use]
    pub fn with_hasher(hasher: H) -> Self {
        assert!(Self::EVICT_WINDOW <= N);
        let direct_sets =
            NonZeroUsize::new(N + (N as f64 * (1. / Self::KEYS_LOAD_FACTOR - 1.)) as usize)
                .unwrap();
        let assoc_ways = NonZeroUsize::new(Self::KEYS_ASSOC_WAYS).unwrap();
        let values = (0..N)
            .map(|_| None)
            .collect::<Vec<_>>()
            .try_into()
            .ok()
            .unwrap();
        Self {
            keys: FixedHashMap::with_hasher(direct_sets, assoc_ways, hasher),
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
        Q: Eq + core::hash::Hash + ?Sized,
    {
        let index = *self.keys.get(key)?;
        Some(self.values[index].as_mut().unwrap().access())
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut final_value_index = None;
        let res = self.keys.get_or_insert(key, |_| {
            let mut least_access_times: Option<usize> = None;
            let mut value_index: Option<usize> = None;
            for i in 0..Self::EVICT_WINDOW {
                let i = self.next_evict.ring_add(i, self.values.len() - 1);
                let init = least_access_times.is_none() && value_index.is_none();
                let empty = least_access_times.is_none() && value_index.is_some();
                let some = least_access_times.is_some() && value_index.is_some();
                let invalid = least_access_times.is_some() && value_index.is_none();
                debug_assert!(!invalid);
                let Some(entry) = &mut self.values[i] else {
                    // performance: This condition probably messes up the branch prediction
                    // if empty {
                    //     continue;
                    // }
                    least_access_times = None;
                    value_index = Some(i);
                    continue;
                };
                if init || (some && entry.times() < least_access_times.unwrap()) {
                    debug_assert!(!empty);
                    least_access_times = Some(entry.times());
                    value_index = Some(i);
                }
                entry.reset_times();
            }
            if Self::EVICT_WINDOW < self.values.len() {
                self.next_evict = self
                    .next_evict
                    .ring_add(Self::EVICT_WINDOW, self.values.len() - 1);
            }
            let value_index = value_index.unwrap();
            final_value_index = Some(value_index);
            value_index
        });
        match res {
            GetOrInsert::Get(&index) => {
                *self.values[index].as_mut().unwrap().access() = value;
            }
            GetOrInsert::Insert((key_index, collided)) => {
                if let Some((_, value_index)) = collided {
                    self.values[value_index] = None;
                }
                let value_index = final_value_index.unwrap();
                let entry = self.values[value_index].take();
                if let Some(entry) = entry {
                    if entry.key_index != key_index {
                        self.keys.remove_entry(entry.key_index);
                    }
                }
                self.values[value_index] = Some(Entry::new(value, key_index));
            }
        }
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
        const N: usize = 1 << 10;

        let mut lru: WeakLru<_, _, 4> = WeakLru::new();
        for i in 0..N {
            if i == N - 1 {
                dbg!(&lru);
            }
            lru.insert(i, i);
            assert_eq!(*lru.get_mut(&i).unwrap(), i);
        }
        dbg!(&lru);

        let mut lru: WeakLru<_, _, 5> = WeakLru::new();
        for i in 0..N {
            lru.insert(i, i);
            assert_eq!(*lru.get_mut(&i).unwrap(), i);
        }
        dbg!(&lru);
    }
}

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    use test::Bencher;

    use crate::sync::tests::RepeatedData;

    use super::*;

    const LRU_SIZE: usize = 1 << 9;
    const DATA_SIZE: usize = 1 << 6;
    const N: usize = 1 << 12;

    macro_rules! weak_lru_insert {
        ($bencher: ident, $lru: ident) => {
            $bencher.iter(|| {
                for i in 0..N {
                    $lru.insert(i, RepeatedData::new(i as _));
                }
            });
        };
    }
    #[bench]
    fn bench_weak_lru(bencher: &mut Bencher) {
        let mut lru: WeakLru<usize, RepeatedData<u8, DATA_SIZE>, LRU_SIZE> = WeakLru::new();
        weak_lru_insert!(bencher, lru);
    }
    #[bench]
    fn bench_weak_lru_hashbrown(bencher: &mut Bencher) {
        let mut lru: WeakLru<
            usize,
            RepeatedData<u8, DATA_SIZE>,
            LRU_SIZE,
            hashbrown::DefaultHashBuilder,
        > = WeakLru::with_hasher(hashbrown::DefaultHashBuilder::default());
        weak_lru_insert!(bencher, lru);
    }
    #[bench]
    fn bench_weak_lru_ahash(bencher: &mut Bencher) {
        let mut lru: WeakLru<usize, RepeatedData<u8, DATA_SIZE>, LRU_SIZE, ahash::RandomState> =
            WeakLru::with_hasher(ahash::RandomState::default());
        weak_lru_insert!(bencher, lru);
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
