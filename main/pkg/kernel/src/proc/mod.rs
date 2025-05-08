mod context;
mod data;
pub mod manager;
mod paging;
mod pid;
mod process;
mod processor;
mod vm;

use boot::BootInfo;
use manager::*;
use process::*;
use vm::*;

use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use xmas_elf::ElfFile;
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
pub fn init(boot_info: &'static BootInfo) {
    let proc_vm = ProcessVm::new(PageTableContext::new()).init_kernel_vm();

    trace!("Init kernel vm: {:#?}", proc_vm);

    // kernel process
    let kproc = Process::new(String::from("kernel"), None, Some(proc_vm), None);
    let app_list = boot_info.loaded_apps.as_ref();
    manager::init(kproc, app_list);

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

pub fn spawn(name: &str) -> Option<ProcessId> {
    let app = x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list()?;
        app_list.iter().find(|&app| app.name.eq(name))
    })?;

    elf_spawn(name.to_string(), &app.elf)
}

pub fn elf_spawn(name: String, elf: &ElfFile) -> Option<ProcessId> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let process_name = name.to_lowercase();
        let parent = Arc::downgrade(&manager.current());
        let pid = manager.spawn(elf, name, Some(parent), None);

        debug!("Spawned process: {}#{}", process_name, pid);
        pid
    });

    Some(pid)
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn current_pid() -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(processor::get_pid)
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

pub fn read(fd: u8, buf: &mut [u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().read(fd, buf))
}

pub fn write(fd: u8, buf: &[u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().write(fd, buf))
}

pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().get_exit_code(pid).is_none())
}

pub fn wait_pid(pid: ProcessId, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if let Some(ret) = manager.get_exit_code(pid) {
            context.set_rax(ret as usize);
        } else {
            context.set_rax(0xBEE);
        }
    })
}

pub fn process_exit(ret: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // DONE: implement this for ProcessManager
        manager.kill_current(ret);
        manager.switch_next(context);
    })
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}

pub fn list_app() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let Some(app_list) = get_process_manager().app_list() else {
            warn!("No app found in list!");
            return;
        };

        let apps = app_list
            .iter()
            .map(|app| app.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        // TODO: print more information like size, entry point, etc.

        info!("App list: {}", apps);
    });
}

pub fn fork(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // DONE: save_current as parent
        // DONE: fork to get child
        // DONE: push to child & parent to ready queue
        // DONE: switch to next process
        let parent = manager.current().pid();
        manager.save_current(context);
        manager.fork();
        manager.push_ready(parent);
        manager.switch_next(context);
    })
}
