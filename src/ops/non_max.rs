use super::opt::Opt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct OptNonMax<T> {
    v: T,
}
impl<T> Opt<T> for OptNonMax<T>
where
    T: num_traits::Bounded + Eq + Copy,
{
    type GetOut = T;
    fn none() -> Self {
        Self { v: T::max_value() }
    }
    fn some(v: T) -> Self {
        assert!(v != T::max_value());
        Self { v }
    }
    fn get(&self) -> Option<Self::GetOut> {
        if self.v == T::max_value() {
            return None;
        }
        Some(self.v)
    }
    fn take(&mut self) -> Option<T> {
        let v = self.get()?;
        *self = Self::none();
        Some(v)
    }
}
impl<T> From<OptNonMax<T>> for Option<T>
where
    T: num_traits::Bounded + Eq + Copy,
{
    fn from(value: OptNonMax<T>) -> Self {
        value.map(|v| Some(v)).unwrap_or(None)
    }
}
impl<T> From<Option<T>> for OptNonMax<T>
where
    T: num_traits::Bounded + Eq + Copy,
{
    fn from(value: Option<T>) -> Self {
        value.map(|v| Self::some(v)).unwrap_or(Self::none())
    }
}
