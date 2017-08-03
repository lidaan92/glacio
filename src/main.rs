extern crate docopt;
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
    config: String,
    addr: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());
    let config = Config::from_path(args.config).unwrap();
    Iron::new(glacio::api::create_router(&config)).http(args.addr).unwrap();
}
