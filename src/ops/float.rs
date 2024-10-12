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
impl<F: Float> Eq for UnitF<F> {}
impl<F: Float> PartialOrd for UnitF<F> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<F: Float> Ord for UnitF<F> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe { self.v.partial_cmp(&other.v).unwrap_unchecked() }
    }
}
impl<F: Debug> Debug for UnitF<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.v.fmt(f)
    }
}
impl<F: core::fmt::Display> core::fmt::Display for UnitF<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.v.fmt(f)
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
        let mut a = [0., 1., 0.1, 0.1].map(|x| UnitF::new(x).unwrap());
        a.sort_unstable();
        assert_eq!(a.map(|x| x.get()), [0., 0.1, 0.1, 1.]);
    }
}
