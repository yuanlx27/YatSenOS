use core::sync::atomic::{ AtomicBool, Ordering };

pub struct SpinLock {
    bolt: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            bolt: AtomicBool::new(false),
        }
    }

    fn try_acquire(&mut self) -> bool {
        self.bolt
            .compare_exchange(
                false,
                true,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
    }
    pub fn acquire(&mut self) {
        // DONE: acquire the lock, spin if the lock is not available
        while ! self.try_acquire() { core::hint::spin_loop(); }
    }

    pub fn release(&mut self) {
        // DONE: release the lock
        self.bolt.store(false, Ordering::Relaxed);
    }
}

impl Default for SpinLock {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Sync for SpinLock {}
