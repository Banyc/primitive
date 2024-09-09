#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OptionNonMax<T> {
    value: T,
}
impl<T> OptionNonMax<T>
where
    T: num_traits::Bounded + Eq,
{
    pub fn some(value: T) -> Option<Self> {
        if value == T::max_value() {
            return None;
        }
        Some(Self { value })
    }
    pub fn none() -> Self {
        let value = T::max_value();
        Self { value }
    }
}
impl<T> OptionNonMax<T>
where
    T: num_traits::Bounded + Eq + Copy,
{
    pub fn get(&self) -> Option<T> {
        if self.value == T::max_value() {
            return None;
        }
        Some(self.value)
    }
}
