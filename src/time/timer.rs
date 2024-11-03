use core::time::Duration;
use std::time::Instant;

use crate::Clear;

#[derive(Debug, Clone)]
pub struct Timer {
    start: Option<Instant>,
}
impl Timer {
    /// Create an cleared timer
    pub fn new() -> Self {
        Self { start: None }
    }

    pub fn restart(&mut self, now: Instant) {
        self.start = Some(now);
    }

    /// Return the start time
    pub fn ensure_started(&mut self, now: Instant) -> Instant {
        match self.start {
            Some(x) => x,
            None => {
                self.start = Some(now);
                now
            }
        }
    }

    /// Return `true` iff the timer sets off
    pub fn ensure_started_and_check(
        &mut self,
        at_least_for: Duration,
        now: Instant,
    ) -> (bool, Duration) {
        let start = self.ensure_started(now);

        // Check the duration condition
        let dur = now.duration_since(start);
        let set_off = at_least_for <= dur;
        (set_off, dur)
    }
}
impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
impl Clear for Timer {
    fn clear(&mut self) {
        self.start = None;
    }
}
