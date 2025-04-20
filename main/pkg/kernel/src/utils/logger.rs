use log::{Level, Metadata, Record};

use owo_colors::OwoColorize;

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // DONE: Configure the logger
    log::set_max_level(log::LevelFilter::Trace);

    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // DONE: Implement the logger with serial output
        //let prefix = match record.level() {
        //    log::Level::Error => "\x1B[1;31mERROR:",
        //    log::Level::Warn => "\x1B[1;33mWARNING:",
        //    log::Level::Info => "\x1B[0;32mINFO:",
        //    log::Level::Debug => "\x1B[0;37mDEBUG:",
        //    log::Level::Trace => "\x1B[0;30mTRACE:",
        //};
        //println!("{} {} \x1B[0;39m", prefix, record.args());

        match record.level() {
            Level::Error => {
                println!("{} {}", "[x]".bold().red(), record.args().red());
            }
            Level::Warn => {
                println!("{} {}", "[!]".bold().yellow(), record.args().yellow());
            }
            Level::Info => {
                println!("{} {}", "[+]".bold().green(), record.args().green());
            }
            Level::Debug => {
                println!("{} {}", "[D]".bold().blue(), record.args().blue());
            }
            Level::Trace => {
                println!("{} {}", "[_]".bold().dimmed(), record.args().dimmed());
            }
        }
    }

    fn flush(&self) {}
}
