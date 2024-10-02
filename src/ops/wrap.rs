pub trait RingSpace
where
    Self: num_traits::Num + PartialOrd + Ord + Copy,
{
    fn ring_add(self, other: Self, max: Self) -> Self {
        assert!(self <= max);
        assert!(other <= max);
        let self_til_end = max - self;
        let no_wrapping = other <= self_til_end;
        if no_wrapping {
            return self + other;
        }
        other - self_til_end - Self::one()
    }
    fn ring_sub(self, other: Self, max: Self) -> Self {
        assert!(self <= max);
        assert!(other <= max);
        if other <= self {
            return self - other;
        }
        let diff = other - self - Self::one();
        max - diff
    }
}
impl<T> RingSpace for T where T: num_traits::Num + PartialOrd + Ord + Copy {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_add() {
        let a = 3;
        assert_eq!(a.ring_add(0, 4), 3);
        assert_eq!(a.ring_add(1, 4), 4);
        assert_eq!(a.ring_add(2, 4), 0);
        assert_eq!(a.ring_add(3, 4), 1);
        assert_eq!(a.ring_add(4, 4), 2);
    }

    #[test]
    fn test_ring_sub() {
        let a = 3;
        assert_eq!(a.ring_sub(0, 4), 3);
        assert_eq!(a.ring_sub(1, 4), 2);
        assert_eq!(a.ring_sub(2, 4), 1);
        assert_eq!(a.ring_sub(3, 4), 0);
        assert_eq!(a.ring_sub(4, 4), 4);
    }
}
