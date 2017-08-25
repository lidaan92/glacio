extern crate glacio;
extern crate iron;

use glacio::Api;
use iron::Iron;

fn main() {
    let api = Api::new();
    println!("Serving glacio-api on http://localhost:3000");
    Iron::new(api).http("localhost:3000").unwrap();
}
