use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

pub trait Capacity: Len {
    #[must_use]
    fn capacity(&self) -> usize;
}

#[allow(clippy::len_without_is_empty)]
pub trait Len {
    #[must_use]
    fn len(&self) -> usize;
}
pub trait LenExt: Len {
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T: Len> LenExt for T {}

pub trait Full: Capacity {
    fn is_full(&self) -> bool {
        self.capacity() == self.len()
    }
}
impl<T: Capacity> Full for T {}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> Len for VecDeque<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> Len for LinkedList<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> Len for HashSet<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> Len for BTreeSet<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<K, V> Len for HashMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<K, V> Len for BTreeMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> Len for BinaryHeap<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> Len for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}
