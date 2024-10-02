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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closes_to() {
        let a: f32 = 1.;
        let b: f32 = 1.000000001;
        assert!(a.closes_to(b));
    }
}
