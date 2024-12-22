use core::{borrow::Borrow, hash::Hash, time::Duration};
use std::{collections::HashMap, time::Instant};

use crate::{
    ops::{len::Len, ord_entry::OrdEntry},
    queue::ord_queue::OrdQueue,
};

#[derive(Debug, Clone)]
pub struct ExpiringHashMap<K, V, Time, Duration> {
    hash_map: HashMap<K, (Time, V)>,
    ord_queue: OrdQueue<OrdEntry<Time, K>>,
    duration: Duration,
}
impl<K, V, Time: Ord, Duration> ExpiringHashMap<K, V, Time, Duration> {
    pub fn new(duration: Duration) -> Self {
        Self {
            hash_map: HashMap::new(),
            ord_queue: OrdQueue::new(),
            duration,
        }
    }
    pub fn insert(&mut self, key: K, value: V, now: Time) -> Option<(V, Time)>
    where
        Time: Copy,
        K: Eq + Hash + Clone,
    {
        match self.hash_map.insert(key.clone(), (now, value)) {
            Some((time, v)) => Some((v, time)),
            None => {
                self.ord_queue.push(OrdEntry {
                    key: now,
                    value: key,
                });
                None
            }
        }
    }
    pub fn cleanup(&mut self, now: Time, mut waste: impl FnMut(K, V, Time))
    where
        K: Eq + Hash + Clone,
        Time: TravelBackInTime<Duration = Duration> + Copy,
    {
        let Some(deadline) = now.travel_back_for(&self.duration) else {
            return;
        };
        while let Some(OrdEntry { key: instant, .. }) = self.ord_queue.peek() {
            if deadline < *instant {
                return;
            }

            let key = self
                .ord_queue
                .pop()
                .expect("We know it is not empty.")
                .value;

            let real_instant = self.hash_map[&key].0;

            if deadline < real_instant {
                self.ord_queue.push(OrdEntry {
                    key: real_instant,
                    value: key,
                });
            } else {
                let (_, value) = self.hash_map.remove(&key).unwrap();
                waste(key, value, real_instant);
            }
        }
    }
    pub fn contains_key<Q>(&mut self, key: &Q, now: Time, waste: impl FnMut(K, V, Time)) -> bool
    where
        K: Borrow<Q> + Eq + Hash + Clone,
        Q: ?Sized + Eq + core::hash::Hash,
        Time: TravelBackInTime<Duration = Duration> + Copy,
    {
        self.cleanup(now, waste);
        self.hash_map.contains_key(key)
    }
    pub fn get_mut<Q>(
        &mut self,
        key: &Q,
        now: Time,
        waste: impl FnMut(K, V, Time),
    ) -> Option<&mut V>
    where
        K: Borrow<Q> + Eq + Hash + Clone,
        Q: ?Sized + Eq + core::hash::Hash,
        Time: TravelBackInTime<Duration = Duration> + Copy,
    {
        self.cleanup(now, waste);
        match self.hash_map.get_mut(key) {
            Some((time, value)) => {
                *time = now;
                Some(value)
            }
            None => None,
        }
    }
    pub fn remove<Q>(&mut self, k: &Q) -> Option<(V, Time)>
    where
        K: Borrow<Q> + Eq + Hash + Clone,
        Q: ?Sized + Eq + core::hash::Hash,
    {
        self.hash_map.remove(k).map(|(t, v)| (v, t))
    }
}
impl<K, V, Time, Duration> Len for ExpiringHashMap<K, V, Time, Duration> {
    fn len(&self) -> usize {
        self.hash_map.len()
    }
}

pub trait TravelBackInTime: Sized {
    type Duration;
    fn travel_back_for(&self, duration: &Self::Duration) -> Option<Self>;
}

macro_rules! impl_travel_back_for {
    () => {
        fn travel_back_for(&self, duration: &Self::Duration) -> Option<Self> {
            self.checked_sub(*duration)
        }
    };
}
impl TravelBackInTime for Instant {
    type Duration = Duration;
    impl_travel_back_for!();
}
macro_rules! same_type_impl_travel_back_in_time {
    ($ty: ident) => {
        impl TravelBackInTime for $ty {
            type Duration = $ty;
            impl_travel_back_for!();
        }
    };
}
same_type_impl_travel_back_in_time!(Duration);
same_type_impl_travel_back_in_time!(u8);
same_type_impl_travel_back_in_time!(u16);
same_type_impl_travel_back_in_time!(u32);
same_type_impl_travel_back_in_time!(u64);
same_type_impl_travel_back_in_time!(u128);
same_type_impl_travel_back_in_time!(i8);
same_type_impl_travel_back_in_time!(i16);
same_type_impl_travel_back_in_time!(i32);
same_type_impl_travel_back_in_time!(i64);
same_type_impl_travel_back_in_time!(i128);
