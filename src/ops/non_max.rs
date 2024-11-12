use super::opt::Opt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct OptNonMax<T> {
    v: T,
}
impl<T> Opt<NonMax<T>> for OptNonMax<T>
where
    T: num_traits::Bounded + Eq + Copy,
{
    type GetOut = T;
    fn none() -> Self {
        Self { v: T::max_value() }
    }
    fn some(v: NonMax<T>) -> Self {
        Self { v: v.get() }
    }
    fn get(&self) -> Option<Self::GetOut> {
        NonMax::new(self.v).map(|v| v.get())
    }
    fn take(&mut self) -> Option<NonMax<T>> {
        let v = self.get()?;
        *self = Self::none();
        Some(unsafe { NonMax::new_unchecked(v) })
    }
}
impl<T> From<OptNonMax<T>> for Option<NonMax<T>>
where
    T: num_traits::Bounded + Eq + Copy,
{
    fn from(value: OptNonMax<T>) -> Self {
        value.map(Some).unwrap_or(None)
    }
}
impl<T> From<Option<NonMax<T>>> for OptNonMax<T>
where
    T: num_traits::Bounded + Eq + Copy,
{
    fn from(value: Option<NonMax<T>>) -> Self {
        value.map(Self::some).unwrap_or(Self::none())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NonMax<T> {
    v: T,
}
impl<T> NonMax<T>
where
    T: num_traits::Bounded + Eq,
{
    pub fn new(v: T) -> Option<Self> {
        if v == T::max_value() {
            return None;
        }
        Some(Self { v })
    }
    /// # Safety
    ///
    /// Make sure input is not maxed out
    pub const unsafe fn new_unchecked(v: T) -> Self {
        Self { v }
    }
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        self.v
    }
}
