pub trait Seq<T> {
    fn as_slice(&self) -> &[T];
}
pub trait SeqMut<T>: Seq<T> {
    fn as_slice_mut(&mut self) -> &mut [T];
}

impl<T> Seq<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> SeqMut<T> for Vec<T> {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> Seq<T> for [T; N] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T, const N: usize> SeqMut<T> for [T; N] {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Seq<T> for &[T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> Seq<T> for &mut [T] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
impl<T> SeqMut<T> for &mut [T] {
    fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq() {
        let mut v = vec![1, 2];
        assert_eq!(Seq::as_slice(&v)[0], 1);
        assert_eq!(SeqMut::as_slice_mut(&mut v)[0], 1);
    }
}
