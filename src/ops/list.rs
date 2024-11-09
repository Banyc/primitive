use core::ops::{Index, IndexMut};

use super::len::Len;

pub trait List<T>: Index<usize, Output = T> + Len {}
pub trait ListMut<T>: IndexMut<usize, Output = T> + Len {}

impl<T> List<T> for Vec<T> {}
impl<T> ListMut<T> for Vec<T> {}
impl<T> List<T> for [T] {}
impl<T> ListMut<T> for [T] {}
impl<T, const N: usize> List<T> for [T; N] {}
impl<T, const N: usize> ListMut<T> for [T; N] {}
