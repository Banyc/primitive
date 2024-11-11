use core::{
    cmp::Ordering,
    ops::{Add, Div, Sub},
};

use num_traits::{Bounded, One, WrappingAdd, WrappingSub, Zero};

pub trait SeqSpace:
    Copy
    + Eq
    + Ord
    + Bounded
    + Zero
    + One
    + Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + WrappingAdd
    + WrappingSub
{
}
impl SeqSpace for u8 {}
impl SeqSpace for u16 {}
impl SeqSpace for u32 {}
impl SeqSpace for u64 {}
impl SeqSpace for u128 {}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct RingSeq<T> {
    v: T,
}

impl<T> RingSeq<T> {
    #[must_use]
    pub const fn new(v: T) -> Self {
        Self { v }
    }
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.v
    }
}

impl<T> RingSeq<T>
where
    T: WrappingAdd,
{
    #[must_use]
    pub fn add(&self, n: T) -> Self {
        let v = self.v.wrapping_add(&n);
        Self { v }
    }
}

impl<T> RingSeq<T>
where
    T: WrappingSub,
{
    #[must_use]
    pub fn sub(&self, n: T) -> Self {
        let v = self.v.wrapping_sub(&n);
        Self { v }
    }
}

impl<T> RingSeq<T>
where
    T: Ord + WrappingSub + Zero + Copy + Bounded + One + Div<Output = T>,
{
    #[must_use]
    pub fn ord_dist(lo: &Self, hi: &Self) -> T {
        match Self::ring_cmp(lo, hi) {
            Ordering::Less => hi.v.wrapping_sub(&lo.v),
            Ordering::Greater => lo.v.wrapping_sub(&hi.v),
            Ordering::Equal => T::zero(),
        }
    }
}

impl<T> RingSeq<T>
where
    T: Zero,
{
    #[must_use]
    pub fn zero() -> Self {
        Self { v: T::zero() }
    }
}

impl<T> RingSeq<T>
where
    T: Eq
        + Sub<Output = T>
        + PartialOrd
        + Ord
        + Copy
        + Bounded
        + One
        + Add<Output = T>
        + Div<Output = T>,
{
    pub fn ring_cmp(&self, other: &Self) -> Ordering {
        match self.v.partial_cmp(&other.v).unwrap() {
            Ordering::Less => {
                let diff = other.v - self.v;
                match diff <= T::max_value() / (T::one() + T::one()) {
                    true => Ordering::Less,
                    false => Ordering::Greater,
                }
            }
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => {
                let diff = self.v - other.v;
                match diff <= T::max_value() / (T::one() + T::one()) {
                    true => Ordering::Greater,
                    false => Ordering::Less,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seq_space_trait() {
        fn test<T: SeqSpace>(v: T) {
            let a = RingSeq::<T>::new(v);
            let b = a.add(T::one());
            let _c = b.sub(T::one());
            let _dist = RingSeq::<T>::ord_dist(&a, &b);
            let _zero = RingSeq::<T>::zero();
        }
        test::<u8>(2);
        test::<u16>(2);
        test::<u32>(2);
        test::<u64>(2);
    }

    #[test]
    fn cmp_wraparound() {
        let a = RingSeq::new(u32::MAX);
        let b = RingSeq::new(u32::MIN);
        assert_eq!(a.ring_cmp(&b), core::cmp::Ordering::Less);
    }

    #[test]
    fn cmp_no_wraparound() {
        let a = RingSeq::new(0);
        let b = RingSeq::new(1);
        assert_eq!(a.ring_cmp(&b), core::cmp::Ordering::Less);
    }

    #[test]
    fn cmp_far() {
        let a = RingSeq::new(0);
        let b = RingSeq::new(i32::MAX as u32);
        let c = RingSeq::new(i32::MAX as u32 + 1);
        assert_eq!(a.ring_cmp(&b), core::cmp::Ordering::Less);
        assert_eq!(c.ring_cmp(&a), core::cmp::Ordering::Less);
    }

    #[test]
    fn add_wraparound() {
        let a = RingSeq::new(u32::MAX);
        let b = a.add(1);
        assert_eq!(b.value(), &0);
    }

    #[test]
    fn add_no_wraparound() {
        let a = RingSeq::new(0);
        let b = a.add(1);
        assert_eq!(b.value(), &1);
    }

    #[test]
    fn sub_wraparound() {
        let a = RingSeq::new(0);
        let b = RingSeq::new(u32::MAX);
        assert_eq!(RingSeq::ord_dist(&a, &b), 1);
    }

    #[test]
    fn sub_zero() {
        let a = RingSeq::new(1);
        let b = RingSeq::new(1);
        assert_eq!(RingSeq::ord_dist(&a, &b), 0);
    }

    #[test]
    fn sub_no_wraparound() {
        let a = RingSeq::new(3);
        let b = RingSeq::new(1);
        assert_eq!(RingSeq::ord_dist(&a, &b), 2);
    }
}
