use std::collections::HashMap;

use crate::queue::fixed_queue::BitQueue;

#[derive(Debug, Clone)]
pub struct Fail {
    points: HashMap<&'static str, BitQueue>,
}
impl Fail {
    #[must_use]
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
        }
    }
    #[must_use]
    pub fn try_fail(&mut self, name: &str) -> bool {
        let Some(events) = self.points.get_mut(name) else {
            return false;
        };
        events.dequeue().unwrap_or(false)
    }
    pub fn set(&mut self, name: &'static str, events: BitQueue) {
        self.points.insert(name, events);
    }
}
impl Default for Fail {
    fn default() -> Self {
        Self::new()
    }
}
