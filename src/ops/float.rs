use core::fmt::Debug;

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
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl<F: Float> Ord for $struct<F> {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
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
#[repr(transparent)]
pub struct UnitR<F> {
    v: R<F>,
}
impl<F: Float> UnitR<F> {
    pub fn new(float: F) -> Option<Self> {
        if !(F::zero()..=F::one()).contains(&float) {
            return None;
        }
        Some(Self { v: R::new(float)? })
    }
}
impl<F> UnitR<F> {
    /// # Safety
    ///
    /// Float must be in the unit interval [0, 1]
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self {
            v: R::new_unchecked(float),
        }
    }
}
impl<F: Copy> UnitR<F> {
    pub const fn get(&self) -> F {
        self.v.get()
    }
}
impl_common_real_number_traits!(UnitR, v);

/// float in \[0, inf)
#[derive(Clone, Copy, PartialEq, Hash)]
#[repr(transparent)]
pub struct NonNegR<F> {
    v: R<F>,
}
impl<F: Float> NonNegR<F> {
    pub fn new(float: F) -> Option<Self> {
        if !(F::zero()..).contains(&float) {
            return None;
        }
        Some(Self { v: R::new(float)? })
    }
}
impl<F> NonNegR<F> {
    /// # Safety
    ///
    /// Float must be in \[0, inf)
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self {
            v: R::new_unchecked(float),
        }
    }
}
impl<F: Copy> NonNegR<F> {
    pub const fn get(&self) -> F {
        self.v.get()
    }
}
impl_common_real_number_traits!(NonNegR, v);

/// float in (0, inf)
#[derive(Clone, Copy, PartialEq, Hash)]
#[repr(transparent)]
pub struct PosR<F> {
    v: R<F>,
}
impl<F: Float> PosR<F> {
    pub fn new(float: F) -> Option<Self> {
        if NonNegR::new(float)?.get() == F::zero() {
            return None;
        }
        Some(Self { v: R::new(float)? })
    }
}
impl<F> PosR<F> {
    /// # Safety
    ///
    /// Float must be in (0, inf)
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self {
            v: R::new_unchecked(float),
        }
    }
}
impl<F: Copy> PosR<F> {
    pub const fn get(&self) -> F {
        self.v.get()
    }
}
impl_common_real_number_traits!(PosR, v);

/// float in (-inf, inf)
#[derive(Clone, Copy, PartialEq, Hash)]
#[repr(transparent)]
pub struct R<F> {
    v: F,
}
impl<F: Float> R<F> {
    pub fn new(float: F) -> Option<Self> {
        if !float.is_finite() {
            return None;
        }
        Some(Self { v: float })
    }
}
impl<F> R<F> {
    /// # Safety
    ///
    /// Float must be in (-inf, inf)
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self { v: float }
    }
}
impl<F: Copy> R<F> {
    pub const fn get(&self) -> F {
        self.v
    }
}
impl_common_real_number_traits!(R, v);

impl<F: Float> From<UnitR<F>> for NonNegR<F> {
    fn from(value: UnitR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}
impl<F: Float> From<UnitR<F>> for R<F> {
    fn from(value: UnitR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}
impl<F: Float> From<NonNegR<F>> for R<F> {
    fn from(value: NonNegR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}
impl<F: Float> From<PosR<F>> for R<F> {
    fn from(value: PosR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}

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
        let mut a = [0., 1., 0.1, 0.1].map(|x| UnitR::new(x).unwrap());
        a.sort_unstable();
        assert_eq!(a.map(|x| x.get()), [0., 0.1, 0.1, 1.]);
    }
}
