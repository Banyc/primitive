use thiserror::Error;

use crate::{ops::len::Len, queue::grow_queue::GrowQueue};

/// [I/O-Free (Sans-I/O)](https://sans-io.readthedocs.io/how-to-sans-io.html)
#[derive(Debug)]
pub struct InnerBuf {
    buf: GrowQueue<u8>,
}
impl InnerBuf {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: GrowQueue::new(),
        }
    }
    pub fn batch_enqueue(&mut self, bytes: &[u8]) {
        self.buf.batch_enqueue(bytes);
    }
    #[must_use]
    pub fn available(&self, additional: usize) -> usize {
        self.buf.len() + additional
    }
    pub fn read_array<const N: usize>(
        &mut self,
        additional: &mut &[u8],
    ) -> Result<[u8; N], NotEnoughBytes> {
        let array = self.copy_array(additional)?;
        self.advance(N, additional);
        Ok(array)
    }
    pub fn copy_array<const N: usize>(
        &mut self,
        additional: &[u8],
    ) -> Result<[u8; N], NotEnoughBytes> {
        let mut array = [0; N];
        self.copy_exact(&mut array, additional)?;
        Ok(array)
    }
    pub fn copy_exact(&mut self, buf: &mut [u8], additional: &[u8]) -> Result<(), NotEnoughBytes> {
        if self.available(additional.len()) < buf.len() {
            return Err(NotEnoughBytes);
        }
        let (a, b) = self.buf.as_slices().unwrap_or((&[], None));
        let b = b.unwrap_or(&[]);
        let mut remaining = buf.len();
        let a_len = a.len().min(remaining);
        remaining -= a_len;
        let b_len = b.len().min(remaining);
        remaining -= b_len;
        let c_len = additional.len().min(remaining);
        let mut start = 0;
        buf[start..start + a_len].copy_from_slice(&a[..a_len]);
        start += a_len;
        buf[start..start + b_len].copy_from_slice(&b[..b_len]);
        start += b_len;
        buf[start..].copy_from_slice(&additional[..c_len]);
        Ok(())
    }
    /// # Panic
    ///
    /// `n` is more than `self.available(additional)`
    pub fn advance(&mut self, n: usize, additional: &mut &[u8]) {
        assert!(n <= self.available(additional.len()));
        let mut remaining = n;
        let buf_len = self.buf.len().min(remaining);
        remaining -= buf_len;
        self.buf.batch_dequeue(buf_len);
        let slice_len = additional.len().min(remaining);
        *additional = &additional[slice_len..];
    }
}
impl Default for InnerBuf {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, Error)]
#[error("not enough bytes")]
pub struct NotEnoughBytes;

pub fn read_array<const N: usize>(bytes: &mut &[u8]) -> Result<[u8; N], NotEnoughBytes> {
    let array = copy_array(bytes)?;
    advance(bytes, N);
    Ok(array)
}
pub fn copy_array<const N: usize>(bytes: &[u8]) -> Result<[u8; N], NotEnoughBytes> {
    let mut array = [0; N];
    copy_exact(&mut array, bytes)?;
    Ok(array)
}
pub fn copy_exact(buf: &mut [u8], bytes: &[u8]) -> Result<(), NotEnoughBytes> {
    if bytes.len() < buf.len() {
        return Err(NotEnoughBytes);
    }
    buf.copy_from_slice(&bytes[..buf.len()]);
    Ok(())
}
/// # Panic
///
/// `n` is more than `buf.len()`
pub fn advance(buf: &mut &[u8], n: usize) {
    assert!(buf.len() <= n);
    *buf = &buf[n..];
}

#[cfg(test)]
pub mod tests {
    use core::{hint::black_box, time::Duration};
    use std::time::Instant;

    use crate::{
        ops::unit::{DurationExt, HumanDuration},
        time::{stopwatch::Stopwatch, timer::Timer},
    };

    use super::*;

    const LENGTH: usize = 64;
    const REPORT_INTERVAL: Duration = Duration::from_millis(500);

    #[test]
    fn test_inner_buf() {
        let mut buf = InnerBuf::new();
        let a: [u8; LENGTH] = (0..LENGTH as u8).collect::<Vec<u8>>().try_into().unwrap();
        buf.batch_enqueue(&a);

        let mut timer = Timer::new();
        let mut watch = Stopwatch::default();
        let mut count = 0;
        let mut batch_count = 0;

        loop {
            watch.start();
            let b = buf.copy_array(&[]).unwrap();
            black_box(b);
            watch.pause();
            count += 1;
            assert_eq!(a, b);
            let now = Instant::now();
            let (set_off, _) = timer.ensure_started_and_check(REPORT_INTERVAL, now);
            if set_off {
                println!("{:.1}", HumanDuration(watch.elapsed().div_u128(count)));
                timer.restart(now);
                if batch_count == 2 {
                    break;
                }
                batch_count += 1;
            }
        }
    }

    #[test]
    #[ignore]
    fn test_alloc() {
        let mut timer = Timer::new();
        let mut watch = Stopwatch::default();
        let mut count = 0;
        let bytes = [0_u8; LENGTH * 2];
        loop {
            watch.start();
            let a = bytes[..LENGTH].to_vec();
            black_box(a);
            watch.pause();
            count += 1;
            let now = Instant::now();
            let (set_off, _) = timer.ensure_started_and_check(REPORT_INTERVAL, now);
            if set_off {
                println!("{:.1}", HumanDuration(watch.elapsed().div_u128(count)));
                timer.restart(now);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_memcpy() {
        let mut timer = Timer::new();
        let mut watch = Stopwatch::default();
        let mut count = 0;
        let bytes = [0_u8; LENGTH * 2];
        loop {
            watch.start();
            let a = memcpy::<LENGTH>(&bytes);
            black_box(a);
            watch.pause();
            count += 1;
            let now = Instant::now();
            let (set_off, _) = timer.ensure_started_and_check(REPORT_INTERVAL, now);
            if set_off {
                println!("{:.1}", HumanDuration(watch.elapsed().div_u128(count)));
                timer.restart(now);
            }
        }
    }
    fn memcpy<const LENGTH: usize>(buf: &[u8]) -> [u8; LENGTH] {
        let mut a = [0; LENGTH];
        a.copy_from_slice(&buf[..LENGTH]);
        a
    }
}
