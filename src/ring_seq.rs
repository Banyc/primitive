use std::{
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
pub struct Seq<T> {
    v: T,
}

impl<T> Seq<T> {
    pub fn new(v: T) -> Self {
        Self { v }
    }

    pub fn value(&self) -> &T {
        &self.v
    }
}

impl<T> Seq<T>
where
    T: WrappingAdd,
{
    pub fn add(&self, n: T) -> Self {
        let v = self.v.wrapping_add(&n);
        Self { v }
    }
}

impl<T> Seq<T>
where
    T: WrappingSub,
{
    pub fn sub(&self, n: T) -> Self {
        let v = self.v.wrapping_sub(&n);
        Self { v }
    }
}

impl<T> Seq<T>
where
    T: Ord + WrappingSub + Zero + Copy + Bounded + One + Div<Output = T>,
{
    pub fn dist(a: &Self, b: &Self) -> T {
        match Self::cmp(a, b) {
            Ordering::Less => b.v.wrapping_sub(&a.v),
            Ordering::Greater => a.v.wrapping_sub(&b.v),
            Ordering::Equal => T::zero(),
        }
    }
}

impl<T> Seq<T>
where
    T: Zero,
{
    pub fn zero() -> Self {
        Self { v: T::zero() }
    }
}

impl<T> PartialOrd for Seq<T>
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Seq<T>
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
    fn cmp(&self, other: &Self) -> Ordering {
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
            let a = Seq::<T>::new(v);
            let b = a.add(T::one());
            let _c = b.sub(T::one());
            let _dist = Seq::<T>::dist(&a, &b);
            let _zero = Seq::<T>::zero();
        }
        test::<u8>(2);
        test::<u16>(2);
        test::<u32>(2);
        test::<u64>(2);
    }

    #[test]
    fn cmp_wraparound() {
        let a = Seq::new(u32::MAX);
        let b = Seq::new(u32::MIN);
        assert!(a < b);
    }

    #[test]
    fn cmp_no_wraparound() {
        let a = Seq::new(0);
        let b = Seq::new(1);
        assert!(a < b);
    }

    #[test]
    fn cmp_far() {
        let a = Seq::new(0);
        let b = Seq::new(i32::MAX as u32);
        let c = Seq::new(i32::MAX as u32 + 1);
        assert!(a < b);
        assert!(c < a);
    }

    #[test]
    fn add_wraparound() {
        let a = Seq::new(u32::MAX);
        let b = a.add(1);
        assert_eq!(b.value(), &0);
    }

    #[test]
    fn add_no_wraparound() {
        let a = Seq::new(0);
        let b = a.add(1);
        assert_eq!(b.value(), &1);
    }

    #[test]
    fn sub_wraparound() {
        let a = Seq::new(0);
        let b = Seq::new(u32::MAX);
        assert_eq!(Seq::dist(&a, &b), 1);
    }

    #[test]
    fn sub_zero() {
        let a = Seq::new(1);
        let b = Seq::new(1);
        assert_eq!(Seq::dist(&a, &b), 0);
    }

    #[test]
    fn sub_no_wraparound() {
        let a = Seq::new(3);
        let b = Seq::new(1);
        assert_eq!(Seq::dist(&a, &b), 2);
    }
}
