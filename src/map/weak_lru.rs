use std::{borrow::Borrow, hash::DefaultHasher, num::NonZeroUsize, time::Instant};

use crate::ops::ring::RingSpace;

use super::fixed_map::FixedHashMap;

#[derive(Debug, Clone)]
pub struct WeakLru<K, V, const N: usize> {
    keys: FixedHashMap<K, usize, DefaultHasher>,
    values: [Option<Entry<V>>; N],
    next_evict: usize,
}
impl<K, V, const N: usize> WeakLru<K, V, N> {
    const EVICT_WINDOW: usize = 4;
    const LOAD_FACTOR: f64 = 0.75;
    #[must_use]
    pub fn new() -> Self {
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
            keys: FixedHashMap::new(hash_map_size),
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
impl<K, V, const N: usize> WeakLru<K, V, N>
where
    K: Eq + core::hash::Hash,
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
        let mut least_access_time: Option<Instant> = None;
        let mut value_index: Option<usize> = None;
        for i in 0..Self::EVICT_WINDOW {
            let i = self.next_evict.ring_add(i, self.values.len() - 1);
            let Some(entry) = &self.values[i] else {
                value_index = Some(i);
                break;
            };
            if let Some(least_access_time) = least_access_time {
                if least_access_time <= entry.last_access {
                    continue;
                }
            }
            least_access_time = Some(entry.last_access);
            value_index = Some(i);
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
    last_access: Instant,
}
impl<V> Entry<V> {
    #[must_use]
    pub fn new(value: V, key_index: usize) -> Self {
        Self {
            value,
            key_index,
            last_access: Instant::now(),
        }
    }
    pub fn access(&mut self) -> &mut V {
        self.last_access = Instant::now();
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
