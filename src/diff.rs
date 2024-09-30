use num_traits::{CheckedAdd, CheckedSub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UDiff<U> {
    Pos(U),
    Neg(U),
    Zero,
}
pub trait UDiffExt: num_traits::Unsigned + CheckedAdd + CheckedSub + Ord {
    fn add_diff(self, diff: UDiff<Self>) -> Option<Self> {
        match diff {
            UDiff::Pos(x) => self.checked_add(&x),
            UDiff::Neg(x) => self.checked_sub(&x),
            UDiff::Zero => todo!(),
        }
    }
    fn sub_diff(self, other: Self) -> UDiff<Self> {
        match self.cmp(&other) {
            std::cmp::Ordering::Less => UDiff::Neg(other - self),
            std::cmp::Ordering::Equal => UDiff::Zero,
            std::cmp::Ordering::Greater => UDiff::Pos(self - other),
        }
    }
}
impl<T> UDiffExt for T where T: num_traits::Unsigned + CheckedAdd + CheckedSub + Ord {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_diff() {
        let a: usize = 0;
        let b: usize = 1;
        let c = a.sub_diff(b);
        assert_eq!(b.add_diff(c).unwrap(), a);
    }
}
