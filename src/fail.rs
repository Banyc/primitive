use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Fail {
    points: HashMap<&'static str, u64>,
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
        let is_active = *events & 1 != 0;
        *events >>= 1;
        is_active
    }
    pub fn set(&mut self, name: &'static str, events: u64) {
        self.points.insert(name, events);
    }
}
impl Default for Fail {
    fn default() -> Self {
        Self::new()
    }
}
