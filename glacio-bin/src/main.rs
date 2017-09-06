extern crate docopt;
extern crate env_logger;
extern crate glacio_http;
extern crate iron;
#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use glacio_http::{Api, Config};
use iron::Iron;

const USAGE: &'static str = "
Glacier research data collection and dissemination.

Usage:
    glacio api <config> <addr>
    glacio heartbeats <config>

Options:
    -h --help           Show this screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_api: bool,
    cmd_heartbeats: bool,
    arg_addr: String,
    arg_config: String,
}

fn main() {
    env_logger::init().unwrap();
    let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());
    if args.cmd_api {
        let api = Api::from_path(args.arg_config).unwrap();
        println!("Serving glacio api on http://{}", args.arg_addr);
        Iron::new(api).http(args.arg_addr).unwrap();
    } else if args.cmd_heartbeats {
        let config = Config::from_path(args.arg_config).unwrap();
        for heartbeat in config.atlas.read_sbd().unwrap() {
            println!("{:?}", heartbeat);
        }
    }
}
