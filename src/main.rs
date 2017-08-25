extern crate glacio;
extern crate iron;

use glacio::Api;
use iron::Iron;

fn main() {
    let path = std::env::args().skip(1).next().unwrap();
    let api = Api::from_path(path).unwrap();
    println!("Serving glacio-api on http://localhost:3000");
    Iron::new(api).http("localhost:3000").unwrap();
}
