extern crate glacio;
extern crate iron;

use iron::Iron;

fn main() {
    Iron::new(glacio::api::create_router()).http("localhost:3000").unwrap();
}
