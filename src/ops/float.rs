use std::fmt::Debug;

use num_traits::Float;

pub trait FloatExt: Float {
    fn closes_to(self, other: Self) -> bool {
        let diff = self - other;
        let rel_tol = Self::from(FLOAT_RELATIVE_TOLERANCE).unwrap();
        let abs_tol = Self::from(FLOAT_ABSOLUTE_TOLERANCE).unwrap();
        let tolerance = Self::max(rel_tol * Self::max(self.abs(), other.abs()), abs_tol);
        diff.abs() <= tolerance
    }
}
impl<T> FloatExt for T where T: Float {}

const FLOAT_RELATIVE_TOLERANCE: f64 = 1e-9; // for big absolute numbers
const FLOAT_ABSOLUTE_TOLERANCE: f64 = 1e-9; // for near-zero numbers

macro_rules! impl_common_real_number_traits {
    ($struct: ident, $value: tt) => {
        impl<F: Float> Eq for $struct<F> {}
        impl<F: Float> PartialOrd for $struct<F> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl<F: Float> Ord for $struct<F> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                unsafe { self.$value.partial_cmp(&other.$value).unwrap_unchecked() }
            }
        }
        impl<F: Debug> Debug for $struct<F> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.$value.fmt(f)
            }
        }
        impl<F: core::fmt::Display> core::fmt::Display for $struct<F> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.$value.fmt(f)
            }
        }
    };
}

/// float in \[0, 1\]
#[derive(Clone, Copy, PartialEq, Hash)]
pub struct UnitF<F> {
    v: F,
}
impl<F: Float> UnitF<F> {
    pub fn new(float: F) -> Option<Self> {
        if !(F::zero()..=F::one()).contains(&float) {
            return None;
        }
        Some(Self { v: float })
    }
    /// # Safety
    ///
    /// Float must be in the unit interval [0, 1]
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self { v: float }
    }
    pub fn get(&self) -> F {
        self.v
    }
}
impl_common_real_number_traits!(UnitF, v);

/// float in \[0, inf)
#[derive(Clone, Copy, PartialEq, Hash)]
pub struct NonNegF<F> {
    v: F,
}
impl<F: Float> NonNegF<F> {
    pub fn new(float: F) -> Option<Self> {
        if !(F::zero()..).contains(&float) {
            return None;
        }
        Some(Self { v: float })
    }
    /// # Safety
    ///
    /// Float must be in \[0, inf)
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self { v: float }
    }
    pub fn get(&self) -> F {
        self.v
    }
}
impl_common_real_number_traits!(NonNegF, v);

/// float in \[0, inf)
#[derive(Clone, Copy, PartialEq, Hash)]
pub struct PosF<F> {
    v: F,
}
impl<F: Float> PosF<F> {
    pub fn new(float: F) -> Option<Self> {
        if !(F::zero()..).contains(&float) {
            return None;
        }
        if float == F::zero() {
            return None;
        }
        Some(Self { v: float })
    }
    /// # Safety
    ///
    /// Float must be in \[0, inf)
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self { v: float }
    }
    pub fn get(&self) -> F {
        self.v
    }
}
impl_common_real_number_traits!(PosF, v);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closes_to() {
        let a: f32 = 1.;
        let b: f32 = 1.000000001;
        assert!(a.closes_to(b));
    }

    #[test]
    fn test_unit_float() {
        let mut a = [0., 1., 0.1, 0.1].map(|x| UnitF::new(x).unwrap());
        a.sort_unstable();
        assert_eq!(a.map(|x| x.get()), [0., 0.1, 0.1, 1.]);
    }
}
