extern crate iron;
extern crate router;
extern crate time;
extern crate itertools;

use std::fmt;
use std::sync::Mutex;
use std::fs::{ OpenOptions, File };
use std::io::Write;

use itertools::Itertools;

use iron::prelude::*;
use iron::{ Handler, AfterMiddleware };
use iron::status;

use iron::typemap::Key;

use router::Router;

fn main() {
    let log_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("requests.log")
                    .unwrap();
    let mut router = Router::new();
    let mut root_handler = Chain::new(StringResponseHandler::new(String::from("Test\n")));
    let log_writer = DiskLogFinalizer::new(log_file);

    root_handler.link_before(request_logger)
                .link_after(log_writer);
    router.get("/", root_handler, "index");
    Iron::new(router).http("localhost:9001").unwrap();
}

struct StringResponseHandler {
    response_message: String
}

impl StringResponseHandler {
    fn new(message: String) -> StringResponseHandler {
        StringResponseHandler {
            response_message: message,
        }
    }
}

impl Handler for StringResponseHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, self.response_message.as_str())))
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

trait Logger {
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

trait Loggable {
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

trait AutoLogger: Logger + Loggable {
    fn autolog(&mut self) {
        let entry = self.to_log_entry();
        self.log(&entry);
    }
}

impl<T> AutoLogger for T where T: Logger + Loggable {}

struct Logs(Vec<String>);

impl Logs {
    fn new() -> Logs {
        Logs(Vec::new())
    }

    fn append(&mut self, entry: &Loggable) {
        let &mut Logs(ref mut log_vec) = self;
        let string_entry = entry.to_log_entry();
        log_vec.push(string_entry);
    }

    fn extract(self) -> Vec<String> {
        let Logs(log_vec) = self;
        log_vec
    }
}

impl Key for Logs {
    type Value = Logs;
}

fn request_logger(req: &mut Request) -> IronResult<()> {
    req.autolog();
    Ok(())
}

struct DiskLogFinalizer {
    log_file: Mutex<File>,
}

impl DiskLogFinalizer {
    fn new(f: File) -> DiskLogFinalizer {
        DiskLogFinalizer {
            log_file: Mutex::new(f),
        }
    }
}

