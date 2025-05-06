use core::alloc::Layout;

use crate::proc::*;
use crate::utils::*;
use crate::memory::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize

    0
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

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
}

pub fn list_process() {
    // FIXME: list all processes
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
