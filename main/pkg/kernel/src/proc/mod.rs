mod context;
mod data;
pub mod manager;
mod paging;
mod pid;
mod process;
mod processor;
mod vm;

use manager::*;
use process::*;
use vm::ProcessVm;
//use crate::memory::PAGE_SIZE;

use alloc::string::String;
use alloc::sync::Arc;
pub use context::ProcessContext;
pub use data::ProcessData;
pub use paging::PageTableContext;
pub use pid::ProcessId;

use x86_64::VirtAddr;
use x86_64::structures::idt::PageFaultErrorCode;
pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init() {
    let proc_vm = ProcessVm::new(PageTableContext::new()).init_kernel_vm();

    trace!("Init kernel vm: {:#?}", proc_vm);

    // kernel process
    let kproc = Process::new(String::from("kernel"), None, Some(proc_vm), None);
    manager::init(kproc);

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // DONE: switch to the next process
        //   - save current process's context
        //   - handle ready queue update
        //   - restore next process's context
        let manager = get_process_manager();
        manager.save_current(context);
        //manager.push_ready(processor::get_pid());
        manager.switch_next(context);
    });
}

pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let entry = VirtAddr::new(entry as usize as u64);
        get_process_manager().spawn_kernel_thread(entry, name, data)
    })
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn current_process_info() {
    debug!("{:#?}", get_process_manager().current());
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // DONE: get current process's environment variable
        get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}
