extern crate iron;
extern crate router;
extern crate iron_building_a_webservice;

use std::fs::OpenOptions;

use iron::prelude::*;
use iron::Handler;
use iron::status;

use router::Router;

use iron_building_a_webservice::logging::{DiskLogFinalizer, request_logger};

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
