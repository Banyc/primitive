use std::sync::Arc;

use super::mutex::SpinMutex;

type Cell<T> = Arc<SpinMutex<Option<T>>>;

pub fn set_once<T>() -> (SetOnceSetter<T>, SetOnceGetter<T>) {
    let cell = Arc::new(SpinMutex::new(None));
    let setter = SetOnceSetter {
        cell: Arc::clone(&cell),
    };
    let getter = SetOnceGetter {
        value: SetOnceGetterInner::Cell(cell),
    };
    (setter, getter)
}

#[derive(Debug)]
pub struct SetOnceSetter<T> {
    cell: Cell<T>,
}
impl<T> SetOnceSetter<T> {
    pub fn set(self, value: T) {
        *self.cell.lock() = Some(value);
    }
}

#[derive(Debug)]
pub struct SetOnceGetter<T> {
    value: SetOnceGetterInner<T>,
}
impl<T> SetOnceGetter<T> {
    pub fn get(&mut self) -> Option<&T> {
        if let SetOnceGetterInner::Cell(cell) = &self.value {
            let value = cell.lock().take()?;
            self.value = SetOnceGetterInner::Local(value);
        }
        let SetOnceGetterInner::Local(value) = &self.value else {
            unreachable!();
        };
        Some(value)
    }
    pub fn into_inner(mut self) -> Result<T, Self> {
        self.get();
        match self.value {
            SetOnceGetterInner::Cell(_) => Err(self),
            SetOnceGetterInner::Local(value) => Ok(value),
        }
    }
}

#[derive(Debug)]
enum SetOnceGetterInner<T> {
    Cell(Cell<T>),
    Local(T),
}
