use std::sync::{Arc, Condvar, Mutex};

use crate::queue::ind_queue::{IndQueue, QueueIndex};

#[derive(Debug)]
pub struct Notify {
    state: Mutex<CriticalNotify>,
}
#[derive(Debug)]
struct CriticalNotify {
    pub wait_queue: IndQueue<Arc<WaitToken>>,
    pub reused_wait_tokens: Vec<Arc<WaitToken>>,
}
impl Notify {
    #[must_use]
    pub const fn new() -> Self {
        let state = CriticalNotify {
            wait_queue: IndQueue::new(),
            reused_wait_tokens: vec![],
        };
        Self {
            state: Mutex::new(state),
        }
    }

    #[must_use]
    pub fn notified(&self) -> Notified<'_> {
        let mut state = self.state.lock().unwrap();
        let token = match state.reused_wait_tokens.pop() {
            Some(token) => {
                token.clear();
                token
            }
            None => Arc::new(WaitToken::new()),
        };
        let index = state.wait_queue.enqueue(token);
        Notified {
            notify: self,
            index,
        }
    }

    pub fn notify_one(&self) {
        let mut state = self.state.lock().unwrap();
        let Some(token) = state.wait_queue.dequeue() else {
            return;
        };
        token.wake();
        state.reused_wait_tokens.push(token);
    }
    pub fn notify_all(&self) {
        let mut state = self.state.lock().unwrap();
        while let Some(token) = state.wait_queue.dequeue() {
            token.wake();
            state.reused_wait_tokens.push(token);
        }
    }
}
impl Default for Notify {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct WaitToken {
    notified: Mutex<bool>,
    blocker: Condvar,
}
impl WaitToken {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            notified: Mutex::new(false),
            blocker: Condvar::new(),
        }
    }
    pub fn clear(&self) {
        *self.notified.lock().unwrap() = false;
    }
    #[must_use]
    pub fn is_notified(&self) -> bool {
        *self.notified.lock().unwrap()
    }
    pub fn wake(&self) {
        *self.notified.lock().unwrap() = true;
        self.blocker.notify_all();
    }
    pub fn wait(&self) {
        loop {
            let notified = self.notified.lock().unwrap();
            if *notified {
                return;
            }
            let notified = self.blocker.wait(notified).expect("poisoned");
            if *notified {
                return;
            }
        }
    }
}

#[derive(Debug)]
pub struct Notified<'a> {
    notify: &'a Notify,
    index: QueueIndex,
}
impl Notified<'_> {
    pub fn wait(&self) {
        let token = {
            let lock = self.notify.state.lock().unwrap();
            let Some(token) = lock.wait_queue.get(self.index) else {
                return;
            };
            Arc::clone(token)
        };
        token.wait();
    }

    #[must_use]
    pub fn is_notified(&self) -> bool {
        let state = self.notify.state.lock().unwrap();
        let Some(token) = state.wait_queue.get(self.index) else {
            return true;
        };
        token.is_notified()
    }
}
impl Drop for Notified<'_> {
    fn drop(&mut self) {
        let mut state = self.notify.state.lock().unwrap();
        state.wait_queue.remove(self.index);
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;

    #[test]
    fn test_notify() {
        let notify = Arc::new(Notify::new());
        for _ in 0..2 {
            let notified = notify.notified();
            let modified = Arc::new(Mutex::new(false));
            std::thread::scope(|s| {
                for _ in 0..2 {
                    let notified = notify.notified();
                    let args = (notified, &modified);
                    s.spawn(move || {
                        let (notified, modified) = args;
                        notified.wait();
                        assert!(*modified.lock().unwrap());
                        dbg!(*modified.lock().unwrap());
                    });
                }
                s.spawn(|| {
                    std::thread::sleep(Duration::from_secs_f64(0.5));
                    *modified.lock().unwrap() = true;
                    notify.notify_all();
                });
                for _ in 0..2 {
                    s.spawn(|| {
                        notified.wait();
                        assert!(*modified.lock().unwrap());
                        dbg!(*modified.lock().unwrap());
                    });
                }
            });
        }
    }
}
