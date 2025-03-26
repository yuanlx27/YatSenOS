use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;

use crate::interrupt::consts::{Interrupts, Irq};

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

pub struct XApic {
    addr: u64,
}

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        XApic { addr }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        unsafe {
            read_volatile((self.addr + reg as u64) as *const u32)
        }
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        unsafe {
            write_volatile((self.addr + reg as u64) as *mut u32, value);
            self.read(0x20);
        }
    }
}

impl LocalApic for XApic {
    /// If this type APIC is supported
    fn support() -> bool {
        // FIXME: Check CPUID to see if xAPIC is supported.
        CpuId::new().get_feature_info().unwrap().has_apic()
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        unsafe {
            // DONE: Enable local APIC; set spurious interrupt vector.
            let mut spiv = self.read(0x0F0);
            spiv |= 0x100;
            spiv &= !0xFF;
            spiv |= Interrupts::IrqBase as u32 + Irq::Spurious as u32;
            self.write(0x0F0, spiv);

            // DONE: The timer repeatedly counts down at bus frequency
            let mut timer = self.read(0x320);
            timer &= !0xFF;
            timer |= Interrupts::IrqBase as u32 + Irq::Timer as u32;
            timer &= !(1 << 16);
            timer |= 1 << 17;
            self.write(0x320, timer);

            // DONE: Disable logical interrupt lines (LINT0, LINT1)
            self.write(0x350, 0x10000);
            self.write(0x360, 0x10000);

            // DONE: Disable performance counter overflow interrupts (PCINT)
            self.write(0x340, 0x10000);

            // DONE: Map error interrupt to IRQ_ERROR.
            let mut lvt_error = self.read(0x370);
            lvt_error &= !0xFF;
            lvt_error |= Interrupts::IrqBase as u32 + Irq::Error as u32;
            self.write(0x370, lvt_error);

            // DONE: Clear error status register (requires back-to-back writes).
            self.write(0x280, 0);
            self.write(0x280, 0);

            // DONE: Ack any outstanding interrupts.
            self.eoi();

            // DONE: Send an Init Level De-Assert to synchronise arbitration ID's.
            self.write(0x310, 0);
            const BCAST: u32 = 1 << 19;
            const INIT: u32 = 5 << 8;
            const TMLV: u32 = 1 << 15; // TM = 1, LV = 0
            self.write(0x300, BCAST | INIT | TMLV); // set ICR 0x300
            const DS: u32 = 1 << 12;
            while self.read(0x300) & DS != 0 {} // wait for delivery status

            // DONE: Enable interrupts on the APIC (but not on the processor).
            self.write(0x080, 0);
        }

        // NOTE: Try to use bitflags! macro to set the flags.
    }

    fn id(&self) -> u32 {
        // NOTE: Maybe you can handle regs like `0x0300` as a const.
        unsafe { self.read(0x0020) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(0x0030) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(0x0310) as u64) << 32 | self.read(0x0300) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(0x0300).get_bit(12) {}
            self.write(0x0310, (value >> 32) as u32);
            self.write(0x0300, value as u32);
            while self.read(0x0300).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(0x00B0, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
