use crate::{non_max::OptionNonMax, Capacity, Clear, Len};

#[derive(Debug, Clone)]
pub struct SparseSet {
    data: Vec<usize>,
    index: Vec<OptionNonMax<usize>>,
}
impl SparseSet {
    /// # Panic
    ///
    /// Capacity is the max number.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        if capacity == usize::MAX {
            panic!("capacity cannot be the max number");
        }
        let index = vec![OptionNonMax::none(); capacity];
        Self {
            data: vec![],
            index,
        }
    }

    /// # Panic
    ///
    /// `Self::capacity() <= value`.
    pub fn insert(&mut self, value: usize) {
        if self.index[value].get().is_some() {
            return;
        }
        let index = self.data.len();
        self.data.push(value);
        self.index[value] = OptionNonMax::some(index).unwrap();
    }

    /// # Panic
    ///
    /// `Self::capacity() <= value`.
    #[must_use]
    pub fn contains(&self, value: usize) -> bool {
        self.index[value].get().is_some()
    }

    /// # Panic
    ///
    /// `Self::capacity() <= value`.
    pub fn remove(&mut self, value: usize) {
        let Some(index) = self.index[value].get() else {
            return;
        };
        self.data.swap_remove(index);
        let Some(&affected_value) = self.data.get(index) else {
            return;
        };
        self.index[affected_value] = OptionNonMax::some(index).unwrap();
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + Clone + '_ {
        self.data.iter().copied()
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = usize> + Clone + 'a {
        self.index
            .iter()
            .zip(&other.index)
            .enumerate()
            .filter_map(|(i, (a, b))| {
                b.get()?;
                a.get()?;
                Some(i)
            })
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = usize> + Clone + 'a {
        let bubbles = self.capacity().abs_diff(other.capacity());
        let (short, long) = match self.capacity().cmp(&other.capacity()) {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => (self, other),
            std::cmp::Ordering::Greater => (other, self),
        };
        let short = short.index.iter().map(|index| index.get());
        let long = long.index.iter().map(|index| index.get());

        short
            .chain(core::iter::repeat(None).take(bubbles))
            .zip(long)
            .enumerate()
            .filter_map(|(i, (a, b))| {
                if a.is_none() && b.is_none() {
                    return None;
                }
                Some(i)
            })
    }
}
impl Len for SparseSet {
    fn len(&self) -> usize {
        self.data.len()
    }
}
impl Capacity for SparseSet {
    fn capacity(&self) -> usize {
        self.index.len()
    }
}
impl Clear for SparseSet {
    fn clear(&mut self) {
        self.data.clear();
        self.index
            .iter_mut()
            .for_each(|index| *index = OptionNonMax::none());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::LenExt;

    use super::*;

    #[test]
    fn test_sparse_set() {
        let mut s = SparseSet::new(3);
        assert!(!s.contains(0));
        s.insert(1);
        assert!(s.contains(1));
        assert_eq!(s.len(), 1);
        s.remove(1);
        assert!(!s.contains(0));
        assert!(s.is_empty());
    }

    #[test]
    fn test_set_ops() {
        let mut a = SparseSet::new(3);
        a.insert(0);
        let mut b = SparseSet::new(4);
        b.insert(0);
        b.insert(1);
        b.insert(3);

        let union = a.union(&b);
        assert_eq!(union.clone().count(), 3);
        let union: HashSet<usize> = HashSet::from_iter(union.clone());
        assert!(union.contains(&0));
        assert!(union.contains(&1));
        assert!(!union.contains(&2));
        assert!(union.contains(&3));

        let intersection = a.intersection(&b);
        assert_eq!(intersection.clone().count(), 1);
        let intersection: HashSet<usize> = HashSet::from_iter(intersection.clone());
        assert!(intersection.contains(&0));
        assert!(!intersection.contains(&1));
        assert!(!intersection.contains(&2));
        assert!(!intersection.contains(&3));
    }
}
