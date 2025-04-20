use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;

use crate::interrupt::consts::{Interrupts, Irq};

// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;
// Local APIC Registers
pub struct LapicRegister;
impl LapicRegister {
    const TPR: u32 = 0x080;
    const SVR: u32 = 0x0F0;
    const ESR: u32 = 0x280;
    const LVT_TIMER: u32 = 0x320;
    const LVT_PCINT: u32 = 0x340;
    const LVT_LINT0: u32 = 0x350;
    const LVT_LINT1: u32 = 0x360;
    const LVT_ERROR: u32 = 0x370;
    const ICR: u32 = 0x380;
    const DCR: u32 = 0x3E0;
}
// Local APIC BitFlags
bitflags! {
    pub struct SpuriousFlags: u32 {
        const ENABLE = 0x00000100;
        const VECTOR = 0x000000FF;
        const VECTOR_IRQ = Interrupts::IrqBase as u32 + Irq::Spurious as u32;
    }
    pub struct LvtFlags: u32 {
        const MASKED = 0x00010000;
        const PERIODIC = 0x00020000;
        const VECTOR = 0x000000FF;
        const VECTOR_IRQ_TIMER = Interrupts::IrqBase as u32 + Irq::Timer as u32;
        const VECTOR_IRQ_ERROR = Interrupts::IrqBase as u32 + Irq::Error as u32;
    }
}

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
        // DONE: Check CPUID to see if xAPIC is supported.
        CpuId::new().get_feature_info().map(|f| f.has_apic()).unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        unsafe {
            // DONE: Enable local APIC; set spurious interrupt vector.
            let mut spiv = SpuriousFlags::from_bits_truncate(self.read(LapicRegister::SVR));
            spiv.insert(SpuriousFlags::ENABLE);
            spiv.remove(SpuriousFlags::VECTOR);
            spiv.insert(SpuriousFlags::VECTOR_IRQ);
            self.write(LapicRegister::SVR, spiv.bits());

            // Set Initial Count.
            self.write(LapicRegister::ICR, 0x00020000);
            // Set Timer Divide.
            self.write(LapicRegister::DCR, 0x0000000B);
            // DONE: The timer repeatedly counts down at bus frequency
            let mut timer = LvtFlags::from_bits_truncate(self.read(LapicRegister::LVT_TIMER));
            timer.remove(LvtFlags::MASKED);
            timer.insert(LvtFlags::PERIODIC);
            timer.remove(LvtFlags::VECTOR);
            timer.insert(LvtFlags::VECTOR_IRQ_TIMER);
            self.write(LapicRegister::LVT_TIMER, timer.bits());

            // DONE: Disable logical interrupt lines (LINT0, LINT1)
            self.write(LapicRegister::LVT_LINT0, LvtFlags::MASKED.bits());
            self.write(LapicRegister::LVT_LINT1, LvtFlags::MASKED.bits());
            // DONE: Disable performance counter overflow interrupts (PCINT)
            self.write(LapicRegister::LVT_PCINT, LvtFlags::MASKED.bits());

            // DONE: Map error interrupt to IRQ_ERROR.
            let mut error = LvtFlags::from_bits_truncate(self.read(LapicRegister::LVT_ERROR));
            error.remove(LvtFlags::MASKED);
            error.remove(LvtFlags::VECTOR);
            error.insert(LvtFlags::VECTOR_IRQ_ERROR);
            self.write(LapicRegister::LVT_ERROR, error.bits());

            // DONE: Clear error status register (requires back-to-back writes).
            self.write(LapicRegister::ESR, 0);
            self.write(LapicRegister::ESR, 0);

            // DONE: Ack any outstanding interrupts.
            self.eoi();

            // DONE: Send an Init Level De-Assert to synchronise arbitration ID's.
            self.set_icr(0x00088500);

            // DONE: Enable interrupts on the APIC (but not on the processor).
            self.write(LapicRegister::TPR, 0);
        }
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
