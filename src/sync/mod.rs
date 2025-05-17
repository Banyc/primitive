pub mod free_list;
pub mod free_u32_list;
pub mod mcast;
pub mod mutex;
pub mod notify;
pub mod seq_lock;
pub mod set_once;
pub mod sync_unsafe_cell;

#[cfg(test)]
pub mod tests {
    use bytemuck::NoUninit;

    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct RepeatedData<T, const DATA_COUNT: usize> {
        values: [T; DATA_COUNT],
    }
    unsafe impl<T: NoUninit, const DATA_COUNT: usize> NoUninit for RepeatedData<T, DATA_COUNT> {}
    impl<T, const DATA_SIZE: usize> RepeatedData<T, DATA_SIZE>
    where
        T: core::fmt::Debug + PartialEq + Eq + Copy,
    {
        pub const fn new(value: T) -> Self {
            Self {
                values: [value; DATA_SIZE],
            }
        }
        pub fn assert(&self) {
            for (i, v) in self.values.iter().enumerate() {
                assert_eq!(&self.values[0], v, "{i}");
            }
        }
        pub const fn get(&self) -> &[T; DATA_SIZE] {
            &self.values
        }
    }
}
