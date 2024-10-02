use num_traits::{CheckedAdd, CheckedSub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diff<U> {
    Pos(U),
    Neg(U),
    Zero,
}
impl<U> Diff<U> {
    pub fn map<V>(self, f: impl FnOnce(U) -> V) -> Diff<V> {
        match self {
            Diff::Pos(x) => Diff::Pos(f(x)),
            Diff::Neg(x) => Diff::Neg(f(x)),
            Diff::Zero => Diff::Zero,
        }
    }
}
pub trait DiffExt: CheckedAdd + CheckedSub + Ord {
    fn add_diff(self, diff: Diff<Self>) -> Option<Self> {
        match diff {
            Diff::Pos(x) => self.checked_add(&x),
            Diff::Neg(x) => self.checked_sub(&x),
            Diff::Zero => Some(self),
        }
    }
    fn sub_diff(self, other: Self) -> Diff<Self> {
        match self.cmp(&other) {
            std::cmp::Ordering::Less => Diff::Neg(other - self),
            std::cmp::Ordering::Equal => Diff::Zero,
            std::cmp::Ordering::Greater => Diff::Pos(self - other),
        }
    }
}
impl<T> DiffExt for T where T: CheckedAdd + CheckedSub + Ord {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff() {
        let a: usize = 0;
        let b: usize = 1;
        let c = a.sub_diff(b);
        assert_eq!(b.add_diff(c).unwrap(), a);
    }
}
