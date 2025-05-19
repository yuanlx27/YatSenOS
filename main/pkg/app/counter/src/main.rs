#![no_std]
#![no_main]

use lib::*;
extern crate lib;

use spin::*;

const THREAD_COUNT: usize = 8;
static mut COUNTER: isize = 0;

static SEMPH: Semaphore = Semaphore::new(0xDEADBEEF);
static SPLOK: Mutex<SpinLock> = Mutex::new(SpinLock::new());

fn main() -> isize {
    let pid = sys_fork();

    if pid == 0 {
        test_semaphore();
    } else {
        sys_wait_pid(pid);
        test_spinlock();
    }

    0
}

fn test_semaphore() {
    SEMPH.init(1);
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });
}

fn test_spinlock() {
    let mut splok = SPLOK.lock();
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            for _ in 0..100 {
                splok.acquire();
                inc_counter();
                splok.release();
            }
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });
}

fn do_counter_inc() {
    for _ in 0..100 {
        // DONE: protect the critical section
        SEMPH.wait();
        inc_counter();
        SEMPH.signal();
    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
fn inc_counter() {
    unsafe {
        delay();
        let mut val = COUNTER;
        delay();
        val += 1;
        delay();
        COUNTER = val;
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
