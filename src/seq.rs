pub fn dyn_vec_init<T>(size: usize, new_value: impl Fn() -> T) -> Vec<T> {
    (0..size).map(|_| new_value()).collect()
}
pub fn dyn_array_init<T, const N: usize>(new_value: impl Fn() -> T) -> [T; N] {
    let res = dyn_vec_init(N, new_value).try_into();
    let Ok(array) = res else { unreachable!() };
    array
}

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
