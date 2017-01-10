use std::fmt;
use std::sync::Mutex;
use std::fs::File;
use std::io::Write;

use itertools::Itertools;

use iron::prelude::*;
use iron::AfterMiddleware;

use iron::typemap::Key;

use time;

pub trait Logger {
    fn log(&mut self, &Loggable);
    
    fn display(&mut self, entry: &fmt::Display) {
        self.log(&format!("{}", entry));
    }

    fn debug(&mut self, entry: &fmt::Debug) {
        self.log(&format!("{:?}", entry));
    }
}

impl<F> Logger for F where F: 'static + Fn(&Loggable) {
    fn log(&mut self, entry: &Loggable) {
        self(entry);
    }
}

impl<'a, 'b> Logger for Request<'a, 'b> {
    fn log(&mut self, entry: &Loggable) {
        let logs = if self.extensions.contains::<Logs>() {
            self.extensions.get_mut::<Logs>().unwrap()
        } else {
            let logs = Logs::new();
            self.extensions.insert::<Logs>(logs);
            self.extensions.get_mut::<Logs>().unwrap()
        };

        let timestamped_entry = format!("[{}] {}", time::now_utc().asctime(), entry.to_log_entry());
        logs.append(&timestamped_entry);
                
    }
}

pub trait Loggable {
    fn to_log_entry(&self) -> String;
}

impl Loggable for String {
    fn to_log_entry(&self) -> String {
        self.clone()
    }
}

impl<'a, 'b> Loggable for Request<'a, 'b> {
    fn to_log_entry(&self) -> String {
        format!("{:?}", self)
    }
}

pub trait AutoLogger: Logger + Loggable {
    fn autolog(&mut self) {
        let entry = self.to_log_entry();
        self.log(&entry);
    }
}

impl<T> AutoLogger for T where T: Logger + Loggable {}

struct Logs(Vec<String>);

impl Logs {
    pub fn new() -> Logs {
        Logs(Vec::new())
    }

    pub fn append(&mut self, entry: &Loggable) {
        let &mut Logs(ref mut log_vec) = self;
        let string_entry = entry.to_log_entry();
        log_vec.push(string_entry);
    }

    pub fn extract(self) -> Vec<String> {
        let Logs(log_vec) = self;
        log_vec
    }
}

impl Key for Logs {
    type Value = Logs;
}

pub fn request_logger(req: &mut Request) -> IronResult<()> {
    req.autolog();
    Ok(())
}

pub struct DiskLogFinalizer {
    log_file: Mutex<File>,
}

impl DiskLogFinalizer {
    pub fn new(f: File) -> DiskLogFinalizer {
        DiskLogFinalizer {
            log_file: Mutex::new(f),
        }
    }
}

impl AfterMiddleware for DiskLogFinalizer {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let entries = match req.extensions.remove::<Logs>() {
            Some(logs) => logs.extract(),
            None => return Ok(res),
        };

        let mut log_file = match self.log_file.lock() {
            Ok(f) => f,
            Err(_) => return Ok(res),
        };

        if let Err(e) = log_file.write(entries
                                .iter()
                                .join("\n")
                                .into_bytes()
                                .as_slice()) {
            // Last ditch cry for help
            println!("Failed to write to log file! {}", e);
        }

        Ok(res)
            
    }
}
