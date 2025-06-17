#![no_std]
#![no_main]

use lib::*;
extern crate lib;

fn main() -> isize {
    let mut current_dir = String::from("/APP");

    loop {
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
                println!("  cat <file>        Display file contents");
                println!("  cd <dir>          Change current directory");
                println!("  ls [dir]          List directory contents");
                println!("  ps                Show process information");
            },
            "cat" => {
                if args.len() < 2 {
                    println!("Usage: cat <file>");
                    continue;
                }

                let path = if args[1].starts_with('/') {
                    // Absolute path
                    String::from(args[1])
                } else {
                    // Relative path
                    if current_dir.ends_with('/') {
                        format!("{}{}", current_dir, args[1])
                    } else {
                        format!("{}/{}", current_dir, args[1])
                    }
                }
                .to_ascii_uppercase();

                let fd = sys_open(path.as_str());

                if fd == 0 {
                    errln!("Invalid path");
                    continue;
                }

                let mut buf = vec![ 0; 3072 ];
                loop {
                    if let Some(bytes) = sys_read(fd, &mut buf) {
                        print!("{}", core::str::from_utf8(&buf[..bytes]).expect("Invalid UTF-8"));
                        if bytes < buf.len() {
                            break;
                        }
                    } else {
                        errln!("Failed to read file");
                        break;
                    }
                }

                sys_close(fd);
            },
            "cd" => {
                if args.len() < 2 {
                    println!("Usage: cd <directory>");
                    continue;
                }

                let path = if args[1].starts_with('/') {
                    // Absolute path
                    String::from(args[1])
                } else {
                    // Relative path
                    format!("{}/{}", &current_dir, args[1])
                }
                .to_ascii_uppercase();

                let mut canonical: Vec<&str> = Vec::new();
                for segment in path.split('/') {
                    match segment {
                        "" | "." => continue,
                        ".." => {
                            if ! canonical.is_empty() {
                                canonical.pop();
                            }
                        },
                        _ => canonical.push(segment),

                    }
                }

                current_dir = String::from("/") + &canonical.join("/");
            },
            "ls" => {
                if args.len() < 2 {
                    sys_list_dir(current_dir.as_str());
                } else {
                    sys_list_dir(args[1]);
                }
            },
            "ps" => sys_stat(),
            _ => {
                println!("Command not found: {}", args[0]);
            },
        }
    }

    0
}

entry!(main);
