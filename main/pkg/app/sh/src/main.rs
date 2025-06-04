#![no_std]
#![no_main]

use lib::*;
extern crate lib;

use alloc::vec::Vec;

fn main() -> isize {
    loop {
        let current_dir = String::from("/");

        print!("> ");

        let input = stdin().read_line();
        let args: Vec<&str> = input.split_whitespace().collect();

        if args.is_empty() {
            continue;
        }

        match args[0] {
            "exit" | "\x04" => {
                break;
            },
            "exec" => {
                if args.len() == 1 {
                    println!("Usage: exec <app>");
                    continue;
                }

                let name = args[1];
                let pid = sys_spawn(name);
                let _ = sys_wait_pid(pid);
            },
            "help" => {
                println!("Available commands:");
                println!("  exec <app>        Execute an application");
                println!("  exit              Exit the shell");
                println!("  help              Show this help message");
                println!("  ls <dir>          List directory contents");
            },
            "ls" => sys_list_dir(current_dir.as_str()),
            "ps" => sys_stat(),
            _ => {
                println!("Command not found: {}", args[0]);
            },
        }
    }

    0
}

entry!(main);
