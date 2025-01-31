use core::convert::Infallible;

/// # Safety
///
/// `pred == false` is an undefined behavior in non-debug builds
pub const unsafe fn assume(pred: bool) {
    if pred {
        return;
    }
    if cfg!(debug_assertions) {
        panic!();
    }
    unreachable!();
}
/// # Safety
///
/// refer to [`assume`]
pub unsafe fn assume_or(pred: bool, panic: impl FnOnce() -> Infallible) {
    if pred {
        return;
    }
    if cfg!(debug_assertions) {
        panic();
    }
    unreachable!();
}
