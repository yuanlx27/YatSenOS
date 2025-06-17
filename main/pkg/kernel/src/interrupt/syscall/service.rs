use core::alloc::Layout;
// use x86_64::VirtAddr;

use crate::proc::*;
use crate::memory::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> u16 {
    if args.arg1 > 0x100 {
        warn!("sys_spawn: path too long");
        return 0;
    }

    let path = match as_user_str(args.arg0, args.arg1) {
        Some(path) => path,
        None => return 0,
    };

    match fs_spawn(path) {
        Some(pid) => pid.0,
        None => {
            warn!("spawn_process: failed to spawn: {}", path);
            0
        }
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // DONE: get buffer and fd by args
    //      - core::slice::from_raw_parts
    // DONE: call proc::write -> isize
    // DONE: return the result as usize
    let buf = match as_user_slice(args.arg1, args.arg2) {
        Some(buf) => buf,
        None => return usize::MAX,
    };

    let fd = args.arg0 as u8;
    write(fd, buf) as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // DONE: just like sys_write
    let buf = match as_user_slice_mut(args.arg1, args.arg2) {
        Some(buf) => buf,
        None => return usize::MAX,
    };

    let fd = args.arg0 as u8;
    read(fd, buf) as usize
}

pub fn sys_get_pid() -> u16 {
    // DONE: get pid from current process
    current_pid().0
}

pub fn sys_wait_pid(args: &SyscallArgs, context: &mut ProcessContext) {
    let pid = ProcessId(args.arg0 as u16);
    wait_pid(pid, context);
}

pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext) {
    match args.arg0 {
        0 => context.set_rax(new_sem(args.arg1 as u32, args.arg2)),
        1 => context.set_rax(remove_sem(args.arg1 as u32)),
        2 => sem_signal(args.arg1 as u32, context),
        3 => sem_wait(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // DONE: exit process with retcode
    process_exit(args.arg0 as isize, context);
}

pub fn list_process() {
    // DONE: list all processes
    print_process_list();
}

pub fn list_dir(args: &SyscallArgs) {
    if args.arg1 > 0x100 {
        warn!("list_dir: path too long");
        return;
    }

    let Some(path) = as_user_str(args.arg0, args.arg1) else {
        warn!("list_dir: path not exist");
        return;
    };

    crate::filesystem::ls(path);
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn sys_fork(context: &mut ProcessContext) {
    fork(context)
}

pub fn sys_open(args: &SyscallArgs) -> usize {
    let path = match as_user_str(args.arg0, args.arg1) {
        Some(path) => path,
        None => return 0,
    };

    match open(path) {
        Some(fd) => fd as usize,
        None => {
            warn!("sys_open: failed to open {path}");
            0
        }
    }
}

pub fn sys_close(args: &SyscallArgs) -> usize {
    close(args.arg0 as u8) as usize
}

pub fn sys_brk(args: &SyscallArgs) -> usize {
    let new_heap_end = if args.arg0 == 0 {
        None
    } else {
        Some(args.arg0)
    };
    brk(new_heap_end)
}
