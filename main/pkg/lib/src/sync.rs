use crate::*;

use core::sync::atomic::{ AtomicBool, Ordering };

// SpinLock {{{

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

// }}}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semaphore {
    // DONE: record the sem key
    key: u32
}

impl Semaphore {
    pub const fn new(key: u32) -> Self {
        Semaphore { key }
    }

    #[inline(always)]
    pub fn init(&self, value: usize) -> bool {
        sys_new_sem(self.key, value)
    }

    // DONE: other functions with syscall...
    #[inline(always)]
    pub fn signal(&self) {
        sys_sem_signal(self.key)
    }

    #[inline(always)]
    pub fn wait(&self) {
        sys_sem_wait(self.key)
    }

    #[inline(always)]
    pub fn free(&self) -> bool {
        sys_rm_sem(self.key)
    }
}

unsafe impl Sync for Semaphore {}

#[macro_export]
macro_rules! semaphore_array {
    [$($x:expr),+ $(,)?] => {
        [ $($crate::Semaphore::new($x),)* ]
    }
}
