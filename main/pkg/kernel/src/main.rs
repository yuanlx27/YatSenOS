#![no_std]
#![no_main]

#[macro_use]
extern crate log;

use core::arch::asm;
use ysos_kernel as ysos;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    error!("Hello World from YatSenOS v2!");
    warn!("Hello World from YatSenOS v2!");
    info!("Hello World from YatSenOS v2!");
    debug!("Hello World from YatSenOS v2!");
    trace!("Hello World from YatSenOS v2!");

    loop {
        for _ in 0..0x10000000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}
