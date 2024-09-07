use crate::Len;

#[derive(Debug, Clone)]
pub struct FreeList<T> {
    free: Vec<usize>,
    data: Vec<Option<T>>,
    count: usize,
}
impl<T> FreeList<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            free: vec![],
            data: vec![],
            count: 0,
        }
    }
}
impl<T> Default for FreeList<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> FreeList<T> {
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index).and_then(|data| data.as_ref())
    }
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index).and_then(|data| data.as_mut())
    }

    #[must_use]
    pub fn insert(&mut self, value: T) -> usize {
        self.count += 1;
        let Some(index) = self.free.pop() else {
            let index = self.data.len();
            self.data.push(Some(value));
            return index;
        };
        self.data[index] = Some(value);
        index
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        let value = self.data.get_mut(index)?.take()?;
        self.count -= 1;
        self.free.push(index);
        Some(value)
    }
}
impl<T> Len for FreeList<T> {
    fn len(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use crate::LenExt;

    use super::*;

    #[test]
    fn test_free_list() {
        let mut l = FreeList::new();
        assert!(l.is_empty());
        let i_0 = l.insert(0);
        assert_eq!(l.len(), 1);
        let i_1 = l.insert(1);
        assert_eq!(l.len(), 2);
        assert_eq!(*l.get(i_0).unwrap(), 0);
        assert_eq!(*l.get(i_1).unwrap(), 1);
        assert_eq!(l.remove(i_0).unwrap(), 0);
        assert!(l.get(i_0).is_none());
        assert!(l.get(i_1).is_some());
        assert_eq!(l.len(), 1);
        assert_eq!(l.remove(i_1).unwrap(), 1);
        assert!(l.get(i_1).is_none());
        assert!(l.is_empty());
    }
}
