use super::consts::*;

use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::structures::idt::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler);
}

pub extern "x86-interrupt" fn clock_handler(_sf: InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if inc_counter() % 0x2 == 0 {
            info!("Tick! @{}", read_counter());
        }
        super::ack();
    });
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[inline]
pub fn read_counter() -> u64 {
    // DONE: load counter value
    COUNTER.load(Ordering::SeqCst) as u64
}

#[inline]
pub fn inc_counter() -> u64 {
    // DONE: read counter value and increase it
    COUNTER.fetch_add(1, Ordering::SeqCst) as u64
}
