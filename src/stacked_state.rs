use core::cell::RefCell;
use std::path::{Path, PathBuf};

pub trait State {
    type Args;
    fn replace(&mut self, args: Self::Args) -> Self::Args;
    fn swap(&mut self, args: &mut Self::Args);
}

#[derive(Debug)]
pub struct StackedState<S> {
    state: RefCell<S>,
}
impl<S> StackedState<S> {
    pub fn new(state: S) -> Self {
        Self {
            state: RefCell::new(state),
        }
    }
    pub fn get(&self) -> &RefCell<S> {
        &self.state
    }

    pub fn push<A>(&self, args: A) -> PushGuard<'_, S, A>
    where
        S: State<Args = A>,
    {
        let prev = self.state.borrow_mut().replace(args);
        PushGuard { cell: self, prev }
    }
}

#[derive(Debug)]
pub struct PushGuard<'a, S, A>
where
    S: State<Args = A>,
{
    cell: &'a StackedState<S>,
    prev: A,
}
impl<S, A> Drop for PushGuard<'_, S, A>
where
    S: State<Args = A>,
{
    fn drop(&mut self) {
        self.cell.state.borrow_mut().swap(&mut self.prev);
    }
}

#[derive(Debug)]
pub struct StackedValueState<T> {
    value: T,
}
impl<T> StackedValueState<T> {
    pub fn get(&self) -> &T {
        &self.value
    }
}
impl<T> State for StackedValueState<T> {
    type Args = T;
    fn replace(&mut self, args: Self::Args) -> Self::Args {
        core::mem::replace(&mut self.value, args)
    }
    fn swap(&mut self, args: &mut Self::Args) {
        core::mem::swap(&mut self.value, args);
    }
}
pub type StackedValue<T> = StackedState<StackedValueState<T>>;
impl<T> StackedValue<T> {
    pub fn new_value(value: T) -> Self {
        let state = StackedValueState { value };
        StackedState::new(state)
    }
}

#[derive(Debug)]
pub struct StackedCwdState {
    cwd: PathBuf,
}
impl State for StackedCwdState {
    type Args = PathBuf;
    fn replace(&mut self, args: Self::Args) -> Self::Args {
        std::env::set_current_dir(&args).unwrap();
        core::mem::replace(&mut self.cwd, args)
    }
    fn swap(&mut self, args: &mut Self::Args) {
        std::env::set_current_dir(&args).unwrap();
        core::mem::swap(&mut self.cwd, args);
    }
}
pub type StackedCwd = StackedState<StackedCwdState>;
impl StackedCwd {
    pub fn new_current() -> Self {
        let cwd = std::env::current_dir().unwrap();
        let state = StackedCwdState { cwd };
        StackedState::new(state)
    }
    pub fn join(&mut self, path: impl AsRef<Path>) -> PushGuard<'_, StackedCwdState, PathBuf> {
        let cwd = self.state.borrow().cwd.join(path);
        self.push(cwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stacked_value() {
        let s = StackedValue::new_value(0);
        {
            let _g1 = s.push(1);
            assert_eq!(*s.get().borrow().get(), 1);
            let _g2 = s.push(2);
            assert_eq!(*s.get().borrow().get(), 2);
        }
        assert_eq!(*s.get().borrow().get(), 0);
    }
}
