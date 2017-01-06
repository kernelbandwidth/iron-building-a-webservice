extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::Handler;
use iron::status;

use router::Router;

fn main() {
    let mut router = Router::new();
    let mut root_handler = Chain::new(StringResponseHandler::new(String::from("Test\n")));

    root_handler.link_before(before_printer)
                .link_after(after_printer);
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

fn before_printer(req: &mut Request) -> IronResult<()> {
    println!("Got request: {:?}", &req);
    Ok(())
}

fn after_printer(_: &mut Request, res: Response) -> IronResult<Response> {
    println!("Sending response: {:?}", &res);
    Ok(res)
}

