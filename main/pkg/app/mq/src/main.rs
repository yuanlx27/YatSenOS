#![no_std]
#![no_main]

use lib::*;
extern crate lib;

static MUTEX: Semaphore = Semaphore::new(0xBABEBABE);
static EMPTY: Semaphore = Semaphore::new(0xBABFBABF);

static mut COUNT: usize = 0;

entry!(main);
fn main() -> isize {
    MUTEX.init(1);
    EMPTY.init(0);

    let mut pids = [0u16; 16];
    // Fork producers and consumers.
    for i in 0..16 {
        let pid = sys_fork();
        if pid == 0 { // Child Branch
            if i % 2 == 0 { producer() } else { consumer() }
        } else { // Parent Branch
            pids[i] = pid;
        }
    }

    // Print information of current processes.
    sys_stat();

    // Wait for all children to exit.
    for pid in pids {
        println!("Waiting for child process #{}", pid);
        sys_wait_pid(pid);
    }

    MUTEX.free();
    EMPTY.free();

    0
}

fn producer() -> ! {
    let pid = sys_get_pid();
    for _ in 0..10 {
        delay();
        // Wait for other IO operations.
        MUTEX.wait();
        // Add a message (simulated by a number).
        unsafe {
            COUNT += 1;
        }
        println!("Process #{pid} produced a message, current count: {}", unsafe { COUNT });
        // Signal on finishing.
        MUTEX.signal();
        // Signal that the queue is not empty.
        EMPTY.signal();
    }
    sys_exit(0);
}

fn consumer() -> ! {
    let pid = sys_get_pid();
    for _ in 0..10 {
        delay();
        // Wait if message queue is empty.
        EMPTY.wait();
        // Wait for other IO operations.
        MUTEX.wait();
        // Remove a message (simulated by a number).
        unsafe {
            COUNT -= 1;
        }
        println!("Process #{pid} consumed a message, current count: {}", unsafe { COUNT });
        // Signal on finishing.
        MUTEX.signal();
    }
    sys_exit(0);
}

#[inline(never)]
#[unsafe(no_mangle)]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}
