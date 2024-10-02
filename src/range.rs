use std::ops::RangeBounds;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RangeAny<T> {
    pub start: core::ops::Bound<T>,
    pub end: core::ops::Bound<T>,
}
impl<T> core::ops::RangeBounds<T> for RangeAny<T> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        self.start.as_ref()
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        self.end.as_ref()
    }
}
impl<T: ToOwned<Owned = T>> From<core::ops::Range<T>> for RangeAny<T> {
    fn from(value: core::ops::Range<T>) -> Self {
        Self {
            start: value.start_bound().map(|x| x.to_owned()),
            end: value.end_bound().map(|x| x.to_owned()),
        }
    }
}
impl<T: ToOwned<Owned = T>> From<core::ops::RangeInclusive<T>> for RangeAny<T> {
    fn from(value: core::ops::RangeInclusive<T>) -> Self {
        Self {
            start: value.start_bound().map(|x| x.to_owned()),
            end: value.end_bound().map(|x| x.to_owned()),
        }
    }
}
impl<T: ToOwned<Owned = T>> From<core::ops::RangeFrom<T>> for RangeAny<T> {
    fn from(value: core::ops::RangeFrom<T>) -> Self {
        Self {
            start: value.start_bound().map(|x| x.to_owned()),
            end: value.end_bound().map(|x| x.to_owned()),
        }
    }
}
impl<T: ToOwned<Owned = T>> From<core::ops::RangeTo<T>> for RangeAny<T> {
    fn from(value: core::ops::RangeTo<T>) -> Self {
        Self {
            start: value.start_bound().map(|x| x.to_owned()),
            end: value.end_bound().map(|x| x.to_owned()),
        }
    }
}
impl<T: ToOwned<Owned = T>> From<core::ops::RangeToInclusive<T>> for RangeAny<T> {
    fn from(value: core::ops::RangeToInclusive<T>) -> Self {
        Self {
            start: value.start_bound().map(|x| x.to_owned()),
            end: value.end_bound().map(|x| x.to_owned()),
        }
    }
}
impl<T> From<core::ops::RangeFull> for RangeAny<T> {
    fn from(_value: core::ops::RangeFull) -> Self {
        Self {
            start: core::ops::Bound::Unbounded,
            end: core::ops::Bound::Unbounded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_any() {
        let range = RangeAny {
            start: core::ops::Bound::Included(0),
            end: core::ops::Bound::Excluded(2),
        };
        assert_eq!(range, (0..2).into());
    }
}
