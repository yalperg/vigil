#![allow(dead_code)]

use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::OnceLock,
};

pub static LOGGER: OnceLock<std::sync::Mutex<Logger>> = OnceLock::new();

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(file: &str) -> anyhow::Result<Logger> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file)?;

        Ok(Logger { file })
    }

    pub fn log(&mut self, message: &str) -> anyhow::Result<()> {
        writeln!(self.file, "{}", message)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let log_message = format!($($arg)*);
        let logger = crate::logger::LOGGER.get_or_init(|| {
            std::sync::Mutex::new(crate::logger::Logger::new("vigil.log").unwrap())
        });
        if let Ok(mut guard) = logger.lock() {
            let _ = guard.log(&log_message);
        }
    }};
}