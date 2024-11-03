use core::cmp::Ordering;

/// [`None`] is the least
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinNoneOptCmp<T>(pub Option<T>);
impl<T: PartialOrd> PartialOrd for MinNoneOptCmp<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (Some(a), Some(b)) => a.partial_cmp(b),
        }
    }
}
impl<T: Ord> Ord for MinNoneOptCmp<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// [`None`] is the greatest
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxNoneOptCmp<T>(pub Option<T>);
impl<T: PartialOrd> PartialOrd for MaxNoneOptCmp<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Greater),
            (Some(_), None) => Some(Ordering::Less),
            (Some(a), Some(b)) => a.partial_cmp(b),
        }
    }
}
impl<T: Ord> Ord for MaxNoneOptCmp<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}
