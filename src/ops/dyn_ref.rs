#[derive(Debug, Copy)]
pub struct DynRef<T, U> {
    value: T,
    convert: fn(&T) -> &U,
}
impl<T, U> DynRef<T, U> {
    pub fn new(value: T, convert: fn(&T) -> &U) -> Self {
        Self { value, convert }
    }
    pub fn convert(&self) -> &U {
        (self.convert)(&self.value)
    }
}
impl<T> DynRef<T, T> {
    pub fn identity(value: T) -> Self {
        Self::new(value, |v| v)
    }
}
impl<T: Clone, U> Clone for DynRef<T, U> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            convert: self.convert,
        }
    }
}
