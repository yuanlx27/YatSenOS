#![no_std]
#![no_main]

use lib::*;
extern crate lib;

static MUTEX: Semaphore = Semaphore::new(0xBADBABE);
static CHOPSTICKS: [Semaphore; 5] = semaphore_array![ 0, 1, 2, 3, 4 ];

entry!(main);
fn main() -> isize {
    MUTEX.init(4);
    for i in 0..5 {
        CHOPSTICKS[i].init(1);
    }

    let mut pids = [0u16; 5];
    // Fork philosophers.
    for i in 0..5 {
        let pid = sys_fork();
        if pid == 0 { // Child Branch
            philosopher(i);
        } else { // Parent Branch
            pids[i] = pid;
        }
    }

    sys_stat();

    for pid in pids {
        println!("Waiting for child process #{}", pid);
        sys_wait_pid(pid);
    }

    MUTEX.free();
    for i in 0..5 {
        CHOPSTICKS[i].free();
    }

    0
}

fn philosopher(id: usize) -> ! {
    let pid = sys_get_pid();

    for _ in 0..0x100 {
        // Think
        println!("Philosopher #{id} (process #{pid}) is thinking");
        delay();

        // Eat
        MUTEX.wait();
        CHOPSTICKS[(id + 0) % 5].wait();
        CHOPSTICKS[(id + 1) % 5].wait();
        println!("Philosopher #{id} (process #{pid}) is eating");
        CHOPSTICKS[(id + 0) % 5].signal();
        CHOPSTICKS[(id + 1) % 5].signal();
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
