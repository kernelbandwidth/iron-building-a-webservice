extern crate iron;
extern crate router;

use std::fmt;

use iron::prelude::*;
use iron::Handler;
use iron::status;

use iron::typemap::Key;

use router::Router;

fn main() {
    let mut router = Router::new();
    let mut root_handler = Chain::new(StringResponseHandler::new(String::from("Test\n")));

    root_handler.link_after(after_middleware);
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

fn after_middleware(req: &mut Request, res: Response) -> IronResult<Response> {
    req.autolog();
    Ok(res)
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
        println!("{}", entry.to_log_entry());
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

impl Key for Box<Logger> {
    type Value = Box<Logger>;
}
