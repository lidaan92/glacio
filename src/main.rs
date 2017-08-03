extern crate docopt;
extern crate env_logger;
extern crate glacio;
extern crate iron;
#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use glacio::api::Config;
use iron::Iron;

const USAGE: &'static str = "
glac.io api server.

Usage:
    glacio-api <config> [--addr=<string>]

Options:
    -h --help           Show this screen.
    --version           Show version.
    --addr=<string>     Server address [default: localhost:3000].
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_config: String,
    flag_addr: String,
}

fn main() {
    env_logger::init().unwrap();
    let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());
    let config = Config::from_path(args.arg_config).unwrap();
    Iron::new(glacio::api::create_router(&config)).http(args.flag_addr).unwrap();
}
