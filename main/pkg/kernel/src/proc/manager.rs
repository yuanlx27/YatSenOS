use super::*;
//use crate::memory::{
//    self, PAGE_SIZE,
//    allocator::{ALLOCATOR, HEAP_SIZE},
//    get_frame_alloc_for_sure,
//};
use alloc::{collections::*, format};
use spin::{Mutex, RwLock};

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>) {
    // DONE: set init process as Running
    init.write().resume();

    // DONE: set processor's current pid to init's pid
    processor::set_pid(init.pid());

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
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

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid()).expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // DONE: update current process's tick count
        self.current().write().tick();
        // DONE: save current process's context
        self.current().write().save(context);
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

    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);

        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();

        // DONE: set the stack frame
        proc.write().init_stack_frame(entry, stack_top);

        let pid = proc.pid();
        // DONE: add to process map
        self.add_proc(pid, proc);
        // DONE: push to ready queue
        self.push_ready(pid);
        // DONE: return new process pid
        pid
    }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
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

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        self.processes
            .read()
            .values()
            .filter(|p| p.read().status() != ProgramStatus::Dead)
            .for_each(|p| output += format!("{}\n", p).as_str());

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
}
