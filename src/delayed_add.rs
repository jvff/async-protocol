use std::sync::atomic::{AtomicUsize, Ordering};

pub struct DelayedAdd<'t> {
    target: &'t AtomicUsize,
    count: usize,
}

impl<'t> DelayedAdd<'t> {
    pub fn new(target: &'t AtomicUsize) -> Self {
        DelayedAdd {
            target,
            count: 0,
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }
}

impl<'t> Drop for DelayedAdd<'t> {
    fn drop(&mut self) {
        self.target.fetch_add(self.count, Ordering::Relaxed);
    }
}
