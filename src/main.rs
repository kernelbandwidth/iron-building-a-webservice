extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;

use router::Router;

fn main() {
    let mut router = Router::new();
    router.get("/", root_handler, "index");
    Iron::new(router).http("localhost:9001").unwrap();
}

fn root_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello Iron!\n")))
}

