use core::{borrow::Borrow, cmp::Ordering, hash::Hash, time::Duration};
use std::{
    collections::{BinaryHeap, HashMap},
    time::Instant,
};

#[derive(Debug, Clone)]
struct HeapValue<K> {
    instant: Instant,
    key: K,
}
impl<K> PartialOrd for HeapValue<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<K> Ord for HeapValue<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant).reverse()
    }
}
impl<K> PartialEq for HeapValue<K> {
    fn eq(&self, other: &Self) -> bool {
        self.instant.eq(&other.instant)
    }
}
impl<K> Eq for HeapValue<K> {}

#[derive(Debug, Clone)]
pub struct ExpiringHashMap<K, V> {
    hash_map: HashMap<K, (Instant, V)>,
    heap: BinaryHeap<HeapValue<K>>,
    duration: Duration,
}
impl<K, V> ExpiringHashMap<K, V> {
    pub fn new(duration: Duration) -> Self {
        Self {
            hash_map: HashMap::new(),
            heap: BinaryHeap::new(),
            duration,
        }
    }
}
impl<K: Eq + Hash + Clone, V> ExpiringHashMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let now = Instant::now();
        match self.hash_map.insert(key.clone(), (now, value)) {
            Some(prev) => Some(prev.1),
            None => {
                self.heap.push(HeapValue { instant: now, key });
                None
            }
        }
    }

    pub fn cleanup(&mut self) {
        let Some(deadline) = Instant::now().checked_sub(self.duration) else {
            return;
        };
        while let Some(HeapValue { instant, .. }) = self.heap.peek() {
            if *instant > deadline {
                return;
            }

            let key = self.heap.pop().expect("We know it is not empty.").key;

            let real_instant = self.hash_map[&key].0;

            if real_instant > deadline {
                self.heap.push(HeapValue {
                    instant: real_instant,
                    key,
                });
            } else {
                self.hash_map.remove(&key);
            }
        }
    }

    pub fn get<Q>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq + std::hash::Hash,
    {
        self.cleanup();
        match self.hash_map.get_mut(key) {
            Some((time, value)) => {
                *time = Instant::now();
                Some(&*value)
            }
            None => None,
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq + std::hash::Hash,
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

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Eq + std::hash::Hash,
    {
        self.hash_map.remove(k).map(|(_time, value)| value)
    }
}
