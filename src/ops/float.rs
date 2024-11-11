use core::{fmt::Debug, marker::PhantomData};

use num_traits::Float;

use super::opt::Opt;

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

macro_rules! impl_fmt_traits {
    ($struct: ident, $value: tt) => {
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
macro_rules! impl_ord_traits {
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
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct OptR<F, W> {
    v: F,
    _wrapper: PhantomData<W>,
}
impl<F: Float, W: WrapNonNan<F>> Opt<W> for OptR<F, W> {
    type GetOut = W;
    fn none() -> Self {
        Self {
            v: F::nan(),
            _wrapper: PhantomData,
        }
    }
    fn some(wrapper: W) -> Self {
        let v = wrapper.get();
        assert!(!v.is_nan());
        Self {
            v,
            _wrapper: PhantomData,
        }
    }
    fn get(&self) -> Option<W> {
        if self.v.is_nan() {
            return None;
        }
        Some(W::new(self.v).unwrap())
    }
    fn take(&mut self) -> Option<W> {
        let res = self.get();
        self.v = F::nan();
        res
    }
}
impl<F: Float, W: WrapNonNan<F>> From<Option<W>> for OptR<F, W> {
    fn from(value: Option<W>) -> Self {
        value.map(|v| Self::some(v)).unwrap_or(Self::none())
    }
}
impl<F: Float, W: WrapNonNan<F>> From<OptR<F, W>> for Option<W> {
    fn from(value: OptR<F, W>) -> Self {
        value.map(|v| v)
    }
}
pub trait WrapNonNan<F>
where
    Self: Sized,
{
    fn new(float: F) -> Option<Self>;
    fn get(&self) -> F;
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
impl_ord_traits!(UnitR, v);
impl_fmt_traits!(UnitR, v);
impl<F: Float> WrapNonNan<F> for UnitR<F> {
    fn new(float: F) -> Option<Self> {
        Self::new(float)
    }
    fn get(&self) -> F {
        self.get()
    }
}

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
impl_ord_traits!(NonNegR, v);
impl_fmt_traits!(NonNegR, v);
impl<F: Float> WrapNonNan<F> for NonNegR<F> {
    fn new(float: F) -> Option<Self> {
        Self::new(float)
    }
    fn get(&self) -> F {
        self.get()
    }
}

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
impl_ord_traits!(PosR, v);
impl_fmt_traits!(PosR, v);
impl<F: Float> WrapNonNan<F> for PosR<F> {
    fn new(float: F) -> Option<Self> {
        Self::new(float)
    }
    fn get(&self) -> F {
        self.get()
    }
}

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
impl_ord_traits!(R, v);
impl_fmt_traits!(R, v);
impl<F: Float> WrapNonNan<F> for R<F> {
    fn new(float: F) -> Option<Self> {
        Self::new(float)
    }
    fn get(&self) -> F {
        self.get()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct NonNanF<F> {
    v: F,
}
impl<F: Float> NonNanF<F> {
    pub fn new(float: F) -> Option<Self> {
        if float.is_nan() {
            return None;
        }
        Some(Self { v: float })
    }
}
impl<F> NonNanF<F> {
    /// # Safety
    ///
    /// Float must not be NAN
    pub const unsafe fn new_unchecked(float: F) -> Self {
        Self { v: float }
    }
}
impl<F: Copy> NonNanF<F> {
    pub const fn get(&self) -> F {
        self.v
    }
}
impl_fmt_traits!(NonNanF, v);
impl<F: Float> WrapNonNan<F> for NonNanF<F> {
    fn new(float: F) -> Option<Self> {
        Self::new(float)
    }
    fn get(&self) -> F {
        self.get()
    }
}

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
impl<F: Float> From<UnitR<F>> for NonNanF<F> {
    fn from(value: UnitR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}
impl<F: Float> From<NonNegR<F>> for R<F> {
    fn from(value: NonNegR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}
impl<F: Float> From<NonNegR<F>> for NonNanF<F> {
    fn from(value: NonNegR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}
impl<F: Float> From<PosR<F>> for R<F> {
    fn from(value: PosR<F>) -> Self {
        unsafe { Self::new_unchecked(value.get()) }
    }
}
impl<F: Float> From<PosR<F>> for NonNanF<F> {
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

    #[test]
    fn test_opt() {
        let mut a = OptR::some(UnitR::new(1.).unwrap());
        assert_eq!(a.get().unwrap(), UnitR::new(1.).unwrap());
        assert_eq!(a.take().unwrap(), UnitR::new(1.).unwrap());
        assert!(a.get().is_none());
    }
}
