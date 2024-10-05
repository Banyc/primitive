use core::{fmt, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HumanDuration(pub Duration);
impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let duration = self.0;
        let seconds = duration.as_secs_f64();
        let minutes = seconds / 60.;
        let hours = minutes / 60.;
        let milliseconds = seconds * 1_000.;
        let microseconds = milliseconds * 1_000.;
        let nanoseconds = microseconds * 1_000.;
        if 1. <= hours {
            hours.fmt(f)?;
            return write!(f, " h");
        }
        if 1. <= minutes {
            minutes.fmt(f)?;
            return write!(f, " min");
        }
        if 1. <= seconds {
            seconds.fmt(f)?;
            return write!(f, " s");
        }
        if 1. <= milliseconds {
            milliseconds.fmt(f)?;
            return write!(f, " ms");
        }
        if 1. <= microseconds {
            microseconds.fmt(f)?;
            return write!(f, " us");
        }
        nanoseconds.fmt(f)?;
        write!(f, " ns")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HumanBytes(pub u64);
impl fmt::Display for HumanBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0 as f64;
        const INTERVAL: usize = 1 << 10;
        let kilobytes = bytes / INTERVAL as f64;
        let megabytes = kilobytes / INTERVAL as f64;
        let gigabytes = megabytes / INTERVAL as f64;
        let terabytes = gigabytes / INTERVAL as f64;
        if 1. <= terabytes {
            terabytes.fmt(f)?;
            return write!(f, " TB");
        }
        if 1. <= gigabytes {
            gigabytes.fmt(f)?;
            return write!(f, " GB");
        }
        if 1. <= megabytes {
            megabytes.fmt(f)?;
            return write!(f, " MB");
        }
        if 1. <= kilobytes {
            kilobytes.fmt(f)?;
            return write!(f, " KB");
        }
        bytes.fmt(f)?;
        write!(f, " B")
    }
}
