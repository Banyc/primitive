use num_traits::{CheckedAdd, CheckedSub};

use super::wrap::{Map, TransposeOption, TransposeResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diff<U> {
    Pos(U),
    Neg(U),
    Zero,
}
impl<U> Diff<U> {
    pub fn flip(self) -> Self {
        match self {
            Self::Pos(x) => Self::Neg(x),
            Self::Neg(x) => Self::Pos(x),
            Self::Zero => todo!(),
        }
    }
}
impl<U> Map<U> for Diff<U> {
    type Wrap<V> = Diff<V>;
    fn map<V>(self, f: impl FnOnce(U) -> V) -> Self::Wrap<V> {
        match self {
            Diff::Pos(x) => Diff::Pos(f(x)),
            Diff::Neg(x) => Diff::Neg(f(x)),
            Diff::Zero => Diff::Zero,
        }
    }
}
impl<U> TransposeOption for Diff<Option<U>> {
    type Inner = U;
    type Wrap<T> = Diff<U>;
    fn transpose_option(self) -> Option<Diff<U>> {
        match self {
            Diff::Pos(x) => x.map(Diff::Pos),
            Diff::Neg(x) => x.map(Diff::Neg),
            Diff::Zero => Some(Diff::Zero),
        }
    }
}
impl<U, E> TransposeResult for Diff<Result<U, E>> {
    type Inner = U;
    type Error = E;
    type Wrap<T> = Diff<U>;
    fn transpose_result(self) -> Result<Self::Wrap<Self::Inner>, Self::Error> {
        match self {
            Diff::Pos(x) => x.map(Diff::Pos),
            Diff::Neg(x) => x.map(Diff::Neg),
            Diff::Zero => Ok(Diff::Zero),
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
            core::cmp::Ordering::Less => Diff::Neg(other - self),
            core::cmp::Ordering::Equal => Diff::Zero,
            core::cmp::Ordering::Greater => Diff::Pos(self - other),
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
