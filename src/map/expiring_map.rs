use core::{borrow::Borrow, hash::Hash, time::Duration};
use std::{collections::HashMap, time::Instant};

use crate::{ops::ord_entry::OrdEntry, queue::ord_queue::OrdQueue};

use super::{
    hash_map::{HashGetMut, HashRemove},
    MapInsert,
};

#[derive(Debug, Clone)]
pub struct ExpiringHashMap<K, V> {
    hash_map: HashMap<K, (Instant, V)>,
    ord_queue: OrdQueue<OrdEntry<Instant, K>>,
    duration: Duration,
}
impl<K, V> ExpiringHashMap<K, V> {
    pub fn new(duration: Duration) -> Self {
        Self {
            hash_map: HashMap::new(),
            ord_queue: OrdQueue::new(),
            duration,
        }
    }
}
impl<K: Eq + Hash + Clone, V> MapInsert<K, V> for ExpiringHashMap<K, V> {
    type Out = Option<V>;
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let now = Instant::now();
        match self.hash_map.insert(key.clone(), (now, value)) {
            Some(prev) => Some(prev.1),
            None => {
                self.ord_queue.insert(OrdEntry {
                    key: now,
                    value: key,
                });
                None
            }
        }
    }
}
impl<K: Eq + Hash + Clone, V> ExpiringHashMap<K, V> {
    pub fn cleanup(&mut self) {
        let Some(deadline) = Instant::now().checked_sub(self.duration) else {
            return;
        };
        while let Some(OrdEntry { key: instant, .. }) = self.ord_queue.peek() {
            if *instant > deadline {
                return;
            }

            let key = self
                .ord_queue
                .pop()
                .expect("We know it is not empty.")
                .value;

            let real_instant = self.hash_map[&key].0;

            if real_instant > deadline {
                self.ord_queue.insert(OrdEntry {
                    key: real_instant,
                    value: key,
                });
            } else {
                self.hash_map.remove(&key);
            }
        }
    }
}
impl<K: Eq + Hash + Clone, V> HashGetMut<K, V> for ExpiringHashMap<K, V> {
    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq + core::hash::Hash,
    {
        self.cleanup();
        match self.hash_map.get_mut(key) {
            Some((time, value)) => {
                *time = Instant::now();
                Some(value)
            }
            None => None,
        }
    }
}
impl<K: Eq + Hash + Clone, V> HashRemove<K, V> for ExpiringHashMap<K, V> {
    fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq + core::hash::Hash,
    {
        self.hash_map.remove(k).map(|(_time, value)| value)
    }
}
