use core::{
    array,
    num::Wrapping,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

#[derive(Debug)]
pub struct IterVacant<'a, const N: usize> {
    list: &'a mut FreeU32List<N>,
    next: u32,
}
impl<const N: usize> Iterator for IterVacant<'_, N> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let curr_index = self.next;
        let next_slot = self
            .list
            .next_vacant
            .get(usize::try_from(curr_index).unwrap())?;
        self.next = next_slot.load(Ordering::Relaxed);
        Some(curr_index)
    }
}
impl<const N: usize> FreeU32List<N> {
    pub fn iter_vacant(&mut self) -> IterVacant<'_, N> {
        let head_u64 = self.head_vacant.load(Ordering::Acquire);
        let head = u64_to_tidx(head_u64);
        IterVacant {
            list: self,
            next: head.index,
        }
    }
}

#[derive(Debug)]
pub struct FreeU32List<const N: usize> {
    head_vacant: AtomicU64,
    next_vacant: [AtomicU32; N],
}
impl<const N: usize> Default for FreeU32List<N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<const N: usize> FreeU32List<N> {
    pub fn new() -> Self {
        let head = TaggedIndex {
            index: 0,
            tag: Tag::default(),
        };
        let head = AtomicU64::new(tidx_to_u64(head));
        let next = array::from_fn(|i| AtomicU32::new(u32::try_from(i + 1).unwrap()));
        Self {
            head_vacant: head,
            next_vacant: next,
        }
    }
    pub fn alloc(&self) -> Option<u32> {
        Some(loop {
            let head_u64 = self.head_vacant.load(Ordering::Acquire);
            let head = u64_to_tidx(head_u64);
            let next_head = self.next_vacant.get(usize::try_from(head.index).unwrap())?;
            let next_head = next_head.load(Ordering::Relaxed);
            let next_head = TaggedIndex {
                index: next_head,
                tag: head.tag.next(),
            };
            let next_head_u64 = tidx_to_u64(next_head);
            if self
                .head_vacant
                .compare_exchange(head_u64, next_head_u64, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                break head.index;
            }
        })
    }
    /// # Safety
    ///
    /// - `val` is from [`Self::alloc()`]
    /// - once freed after [`Self::alloc()`], `val` will not be freed anymore unless it has been pulled from [`Self::alloc()`] later
    pub unsafe fn free(&self, val: u32) {
        loop {
            let head_u64 = self.head_vacant.load(Ordering::Acquire);
            let head = u64_to_tidx(head_u64);
            self.next_vacant[usize::try_from(val).unwrap()].store(head.index, Ordering::Relaxed);
            let new_head = TaggedIndex {
                index: val,
                tag: head.tag.next(),
            };
            let new_head_u64 = tidx_to_u64(new_head);
            if self
                .head_vacant
                .compare_exchange(head_u64, new_head_u64, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }
}
#[cfg(test)]
#[test]
fn test_basics() {
    let l: FreeU32List<2> = FreeU32List::new();
    let val_1 = l.alloc().unwrap();
    let val_2 = l.alloc().unwrap();
    assert!(l.alloc().is_none());
    unsafe { l.free(val_1) };
    let val_3 = l.alloc().unwrap();
    assert_eq!(val_1, val_3);
    unsafe { l.free(val_2) };
    let val_4 = l.alloc().unwrap();
    assert_eq!(val_2, val_4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TaggedIndex {
    pub index: u32,
    /// solve ABA problem
    pub tag: Tag,
}
fn tidx_to_u64(head: TaggedIndex) -> u64 {
    let index: u64 = head.index.into();
    let tag: u32 = head.tag.into();
    let tag: u64 = tag.into();
    index | tag << u32::BITS
}
fn u64_to_tidx(val: u64) -> TaggedIndex {
    let index: u32 = val as u32;
    let tag: u32 = (val >> u32::BITS) as u32;
    TaggedIndex {
        index,
        tag: tag.into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tag {
    val: Wrapping<u32>,
}
impl Default for Tag {
    fn default() -> Self {
        Self { val: Wrapping(0) }
    }
}
impl Tag {
    pub fn next(&self) -> Self {
        Self {
            val: self.val + Wrapping(1),
        }
    }
}
impl From<u32> for Tag {
    fn from(value: u32) -> Self {
        Self {
            val: Wrapping(value),
        }
    }
}
impl From<Tag> for u32 {
    fn from(value: Tag) -> Self {
        value.val.0
    }
}
