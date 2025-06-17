use syscall_def::Syscall;

#[inline(always)]
pub fn sys_write(fd: u8, buf: &[u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Write,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_read(fd: u8, buf: &mut [u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Read,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_wait_pid(pid: u16) -> isize {
    // DONE: try to get the return value for process
    //        loop until the process is finished
    loop {
        let ret = syscall!(Syscall::WaitPid, pid as u64);

        if ret != 0xBEE {
            return ret as isize;
        }

        core::hint::spin_loop();
    }
}

#[inline(always)]
pub fn sys_list_dir(root: &str) {
    syscall!(Syscall::ListDir, root.as_ptr() as u64, root.len() as u64);
}

#[inline(always)]
pub fn sys_stat() {
    syscall!(Syscall::Stat);
}

#[inline(always)]
pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    syscall!(Syscall::Allocate, layout as *const _) as *mut u8
}

#[inline(always)]
pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) -> usize {
    syscall!(Syscall::Deallocate, ptr, layout as *const _)
}

#[inline(always)]
pub fn sys_fork() -> u16 {
    let pid = syscall!(Syscall::Fork);
    pid as u16
}

#[inline(always)]
pub fn sys_spawn(path: &str) -> u16 {
    syscall!(Syscall::Spawn, path.as_ptr() as u64, path.len() as u64) as u16
}

#[inline(always)]
pub fn sys_get_pid() -> u16 {
    syscall!(Syscall::GetPid) as u16
}

#[inline(always)]
pub fn sys_exit(code: isize) -> ! {
    syscall!(Syscall::Exit, code as u64);
    unreachable!("This process should be terminated by now.")
}

#[inline(always)]
pub fn sys_new_sem(key: u32, value: usize) -> bool {
    syscall!(Syscall::Sem, 0, key as u64, value) == 0
}

#[inline(always)]
pub fn sys_rm_sem(key: u32) -> bool {
    syscall!(Syscall::Sem, 1, key as u64) == 0
}

#[inline(always)]
pub fn sys_sem_signal(key: u32) {
    _ = syscall!(Syscall::Sem, 2, key as u64)
}

#[inline(always)]
pub fn sys_sem_wait(key: u32) {
    _ = syscall!(Syscall::Sem, 3, key as u64)
}

#[inline(always)]
pub fn sys_open(path: &str) -> u8 {
    syscall!(Syscall::Open, path.as_ptr() as u64, path.len() as u64) as u8
}

#[inline(always)]
pub fn sys_close(fd: u8) -> bool {
    syscall!(Syscall::Close, fd as u64) != 0
}

#[inline(always)]
pub fn sys_brk(addr: Option<usize>) -> Option<usize> {
    const BRK_FAILED: usize = !0;
    match syscall!(Syscall::Brk, addr.unwrap_or(0)) {
        BRK_FAILED => None,
        ret => Some(ret),
    }
}
