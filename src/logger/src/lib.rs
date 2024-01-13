use std::{cell::RefCell, time::Duration};

use ic_cdk::{api::time, print};

type LogDateTime = String;
type LogMessage = String;

pub type Logs = Vec<(LogDateTime, LogMessage)>;

thread_local! {
  /* flexible */ static LOGGER: RefCell<Logger> = RefCell::new(Logger::new());
}

struct Logger {
    logs: Logs,
}

impl Logger {
    fn new() -> Self {
        Logger { logs: Logs::new() }
    }

    fn log(&mut self, message: &str) {
        let time_ns = Duration::from_nanos(time());
        let utc_datetime = utc_dt::UTCDatetime::from(time_ns);
        self.logs
            .push((utc_datetime.as_iso_datetime(Some(3)), message.to_string()))
    }
}

pub fn get_logs() -> Logs {
    LOGGER.with(|logger| logger.borrow().logs.clone())
}

pub fn log(message: &str) {
    print(message);
    LOGGER.with(|logger| {
        logger.borrow_mut().log(message);
    });
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        log(&format!($($arg)*));
    }
}
