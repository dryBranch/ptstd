pub use log::{debug, error, info, trace, warn};

use chrono::Local;
use log::{Level, LevelFilter, Log, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

/// 简单日志
/// 
/// # Example
/// ```no_run
/// SLog::init().unwrap();
/// set_max_level(LevelFilter::Info);
/// 
/// trace!("some trace");
/// debug!("some debug");
/// info!("some info");
/// warn!("some warn");
/// error!("some error");
/// ```
pub struct SLog {
    pub destination: LogDestination,
}

/// 日志输出的位置
/// - Console: 输出到标准输出
/// - File: 输出到文件
/// - Network: 输出到网络(TODO)
pub enum LogDestination {
    Console,
    File(Mutex<File>),
    Network,
}

impl SLog {
    pub fn new(dst: LogDestination) -> SLog {
        SLog { destination: dst }
    }

    pub fn init_with(dst: LogDestination, max_level: LevelFilter) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(SLog::new(dst))).map(|()| log::set_max_level(max_level))
    }

    #[inline]
    pub fn init() -> Result<(), SetLoggerError> {
        Self::init_with(LogDestination::Console, LevelFilter::Trace)
    }

    #[inline]
    pub fn init_with_file(f: File, max_level: LevelFilter) -> Result<(), SetLoggerError> {
        Self::init_with(LogDestination::File(Mutex::new(f)), max_level)
    }

    pub fn init_with_filename(fname: &str, max_level: LevelFilter) -> Result<(), SetLoggerError> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(fname)
            .unwrap(); // must crash?
        Self::init_with_file(f, max_level)
    }
}

impl Log for SLog {
    // 这个函数是为了判断级别，避免无用记录的开销
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    #[allow(clippy::unused_io_amount)]
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now();
            let msg = format!(
                "{:<36} {:>5} {} -- {}\n",
                now.to_string(),
                record.level(),
                record.target(),
                record.args()
            );

            match &self.destination {
                LogDestination::Console => print!("{msg}"),
                LogDestination::File(f) => {
                    let mut f = f.lock().unwrap();
                    f.write(msg.as_bytes()).unwrap();
                },
                LogDestination::Network => (),
            };
        }
    }

    fn flush(&self) {}
}


#[cfg(test)]
mod tests {
    use log::set_max_level;

    use super::*;

    #[test]
    fn basic() {
        SLog::init().unwrap();
        set_max_level(LevelFilter::Info);

        trace!("some trace");
        debug!("some debug");
        info!("some info");
        warn!("some warn");
        error!("some error");
    }

    #[test]
    fn basic_file() {
        SLog::init_with_filename("test.log", LevelFilter::Trace).unwrap();
        set_max_level(LevelFilter::Debug);

        trace!("some trace");
        debug!("some debug");
        info!("some info");
        warn!("some warn");
        error!("some error");
        debug!(target: "my_target", "a {} event", "log");
    }
}
