use x86_64::structures::idt::*;
use crate::input::*;
use crate::serial::*;
use super::consts::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // DONE: receive character from uart 16550, put it into INPUT_BUFFER
    if let Some(byte) = get_serial_for_sure().receive() {
        push_key(byte);
    };
}
