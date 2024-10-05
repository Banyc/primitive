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
            return write!(f, "{hours:.2} h");
        }
        if 1. <= minutes {
            return write!(f, "{minutes:.2} min");
        }
        if 1. <= seconds {
            return write!(f, "{seconds:.2} s");
        }
        if 1. <= milliseconds {
            return write!(f, "{milliseconds:.2} ms");
        }
        if 1. <= microseconds {
            return write!(f, "{microseconds:.2} us");
        }
        write!(f, "{nanoseconds:.2} ns")
    }
}
