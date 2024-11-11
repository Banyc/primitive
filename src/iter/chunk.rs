use core::mem::MaybeUninit;

use crate::ops::slice::dyn_vec_init;

pub trait Chunks: Iterator + Sized {
    fn static_chunks<T, const CHUNK_SIZE: usize>(self, for_each: impl FnMut(&[T]))
    where
        Self: Iterator<Item = T>,
    {
        let mut tray = [const { MaybeUninit::uninit() }; CHUNK_SIZE];
        self.chunks(&mut tray, for_each);
    }
    fn dyn_chunks<T, const N: usize>(self, chunk_size: usize, for_each: impl FnMut(&[T]))
    where
        Self: Iterator<Item = T>,
    {
        let mut tray = dyn_vec_init(chunk_size, || MaybeUninit::uninit());
        self.chunks(&mut tray, for_each);
    }
    fn chunks<T>(mut self, tray: &mut [MaybeUninit<T>], mut for_each: impl FnMut(&[T]))
    where
        Self: Iterator<Item = T>,
    {
        let mut i = 0;
        loop {
            let v = self.next();
            let is_end = v.is_none();
            if let Some(v) = v {
                tray[i] = MaybeUninit::new(v);
                i += 1;
                if i < tray.len() {
                    continue;
                }
            }
            if i != 0 {
                let tray = &tray[..i];
                let tray = unsafe { core::mem::transmute::<&[MaybeUninit<T>], &[T]>(tray) };
                for_each(tray);
                i = 0;
            }
            if is_end {
                break;
            }
        }
    }
}
impl<T> Chunks for T where T: Iterator {}
#[cfg(test)]
#[test]
fn test_chunks() {
    {
        let mut buf = vec![];
        let a: [usize; 3] = [0, 1, 2];
        a.iter()
            .static_chunks::<_, 2>(|tray| buf.push(tray.iter().copied().sum::<usize>()));
        assert_eq!(&buf, &[1, 2]);
    }
    {
        let mut buf = vec![];
        let a: [usize; 4] = [0, 1, 2, 3];
        a.iter()
            .static_chunks::<_, 2>(|tray| buf.push(tray.iter().copied().sum::<usize>()));
        assert_eq!(&buf, &[1, 5]);
    }
}
