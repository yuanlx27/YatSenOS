#![no_std]
#![no_main]

use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

#[macro_use]
extern crate log;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    info!("Hello World from YatSenOS v2!");

    loop {
        let input = input::get_line();

        match input.trim() {
            "exit" => break,
            _ => {
                println!("🤪: no such command!");
            }
        }
    }

    ysos::shutdown();
}
