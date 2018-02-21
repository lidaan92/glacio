#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate glacio_http;
extern crate iron;

fn main() {
    use glacio_http::{Api, Config};
    use iron::Iron;
    use clap::App;

    env_logger::init().unwrap();
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    if let Some(matches) = matches.subcommand_matches("api") {
        let api = Api::from_path(matches.value_of("CONFIG").unwrap()).unwrap();
        let addr = matches.value_of("ADDR").unwrap();
        println!("Serving glacio api on http://{}", addr);
        Iron::new(api).http(addr).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("heartbeats") {
        let config = Config::from_path(matches.value_of("CONFIG").unwrap()).unwrap();
        for heartbeat in config.atlas.read_sbd().unwrap() {
            if heartbeat.is_ok() {
                println!("{:?}", heartbeat.unwrap());
            }
        }
    }
}
