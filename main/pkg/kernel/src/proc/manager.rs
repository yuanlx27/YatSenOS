use super::*;
use crate::memory::*;

use boot::AppListRef;

use alloc::format;
use alloc::collections::*;
use alloc::sync::Weak;
use spin::{Mutex, RwLock};

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list: AppListRef) {
    // DONE: set init process as Running
    init.write().resume();
    // DONE: set processor's current pid to init's pid
    processor::set_pid(init.pid());

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    wait_queue: Mutex<BTreeMap<ProcessId, BTreeSet<ProcessId>>>,
    app_list: AppListRef,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(VecDeque::new()),
            wait_queue: Mutex::new(BTreeMap::new()),
            app_list,
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    pub fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    pub fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn app_list(&self) -> AppListRef {
        self.app_list
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid()).expect("No current process")
    }

    #[inline]
    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.current().read().read(fd, buf)
    }
    #[inline]
    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.current().read().write(fd, buf)
    }

    pub fn open(&self, path: &str) -> Option<u8> {
        let stream = match get_rootfs().open_file(path) {
            Ok(file) => Resource::File(file),
            Err(_) => return None,
        };

        let fd = self.current().write().open(stream);
        Some(fd)
    }
    pub fn close(&self, fd: u8) -> bool {
        if fd < 3 {
            false
        } else {
            self.current().write().close(fd)
        }
    }

    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, parent, proc_vm, proc_data);

        let mut inner = proc.write();
        // DONE: load elf to process pagetable
        // DONE: alloc new stack for process
        // DONE: mark process as ready
        inner.pause();
        inner.load_elf(elf);
        inner.init_stack_frame(
            VirtAddr::new(elf.header.pt2.entry_point()),
            VirtAddr::new(super::stack::STACK_INIT_TOP),
        );
        drop(inner);

        trace!("New {:#?}", &proc);

        let pid = proc.pid();
        // DONE: something like kernel thread
        self.add_proc(pid, proc);
        self.push_ready(pid);

        pid
    }

    pub fn save_current(&self, context: &ProcessContext) {
        let proc = self.current();
        let pid = proc.pid();

        let mut inner = proc.write();
        // DONE: update current process's tick count
        inner.tick();
        // DONE: save current process's context
        inner.save(context);

        let status = inner.status();
        drop(inner);

        if status != ProgramStatus::Dead {
            self.push_ready(pid);
        } else {
            debug!("Process #{} {:#?} is dead.", pid, proc);
        }
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {
        // DONE: fetch the next process from ready queue
        let mut next_pid = processor::get_pid();

        // DONE: check if the next process is ready, continue to fetch if not ready
        while let Some(pid) = self.ready_queue.lock().pop_front() {
            let proc = self.get_proc(&pid).unwrap();

            if proc.read().is_ready() {
                next_pid = pid;
                break;
            }
        }

        // DONE: restore next process's context
        let next_proc = self.get_proc(&next_pid).unwrap();
        next_proc.write().restore(context);

        // DONE: update processor's current pid
        processor::set_pid(next_pid);

        // DONE: return next process's pid
        next_pid
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let Some(proc) = self.get_proc(&pid) else {
            error!("Process #{} not found.", pid);
            return;
        };

        if proc.read().is_dead() {
            error!("Process #{} is already dead.", pid);
            return;
        }

        if let Some(pids) = self.wait_queue.lock().remove(&pid) {
            for pid in pids {
                self.wake_up(pid, Some(ret));
            }
        }

        proc.kill(ret);
    }
    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn get_exit_code(&self, pid: ProcessId) -> Option<isize> {
        self.get_proc(&pid).and_then(|p| p.read().exit_code())
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // DONE: handle page fault
        if err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            error!("Page Fault! Protection Violation at {:#x}", addr);
            return false;
        }
        if err_code.contains(PageFaultErrorCode::MALFORMED_TABLE) {
            error!("Page Fault! Malformed Table at {:#x}", addr);
            return false;
        }
        if err_code.contains(PageFaultErrorCode::SHADOW_STACK) {
            error!("Page Fault! Shadow Stack access at {:#x}", addr);
            return false;
        }
        if err_code.contains(PageFaultErrorCode::SGX) {
            error!("Page Fault! Software Guard Extensions violation at {:#x}", addr);
            return false;
        }
        if err_code.contains(PageFaultErrorCode::RMP) {
            error!("Page Fault! Restricted Memory Protection violation at {:#x}", addr);
            return false;
        }

        let proc = self.current();
        trace!("Page Fault! Checking if {:#x} is on current process's stack", addr);

        if proc.pid() == KERNEL_PID {
            info!("Page Fault on kernel at {:#x}", addr);
        }

        proc.write().handle_page_fault(addr);

        true
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  |  Memory  | Status\n");

        self.processes
            .read()
            .values()
            .filter(|p| p.read().status() != ProgramStatus::Dead)
            .for_each(|p| output += format!("{p}\n").as_str());

        // TODO: print memory usage of kernel heap

        let alloc = get_frame_alloc_for_sure();
        let frames_used = alloc.frames_used();
        let frames_recycled = alloc.frames_recycled();
        let frames_total = alloc.frames_total();

        let used = (frames_used - frames_recycled) * PAGE_SIZE as usize;
        let total = frames_total * PAGE_SIZE as usize;

        output += &format_usage("Memory", used, total);
        drop(alloc);

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }

    pub fn wait_pid(&self, pid: ProcessId) {
        let mut wait_queue = self.wait_queue.lock();
        // DONE: push the current process to the wait queue
        let entry = wait_queue.entry(pid).or_default();
        entry.insert(processor::get_pid());
    }

    pub fn fork(&self) {
        // DONE: get current process
        // DONE: fork to get child
        // DONE: add child to process list
        let proc = self.current().fork();
        let pid = proc.pid();
        self.add_proc(pid, proc);
        self.push_ready(pid);

        // FOR DBG: maybe print the process ready queue?
        debug!("Ready Queue: {:?}", self.ready_queue.lock());
    }

    /// Block the process with the given pid
    pub fn block(&self, pid: ProcessId) {
        if let Some(proc) = self.get_proc(&pid) {
            // DONE: set the process as blocked
            proc.write().block();
        }
    }
    /// Wake up the process with the given pid
    ///
    /// If `ret` is `Some`, set the return value of the process
    pub fn wake_up(&self, pid: ProcessId, ret: Option<isize>) {
        if let Some(proc) = self.get_proc(&pid) {
            let mut inner = proc.write();
            if let Some(ret) = ret {
                // DONE: set the return value of the process
                inner.set_return(ret as usize);
            }
            // DONE: set the process as ready
            // DONE: push to ready queue
            inner.pause();
            self.push_ready(pid);
        }
    }
}

// A helper function to format memory usage
fn format_usage(name: &str, used: usize, total: usize) -> String {
    let (used_float, used_unit) = humanized_size(used as u64);
    let (total_float, total_unit) = humanized_size(total as u64);

    format!(
        "{:<6} : {:>6.*} {:>3} / {:>6.*} {:>3} ({:>5.2}%)\n",
        name,
        2,
        used_float,
        used_unit,
        2,
        total_float,
        total_unit,
        used as f32 / total as f32 * 100.0
    )
}
