//! Handle ATLAS requests.

use atlas::{Config, Status};
use iron::{IronResult, Request, Response};
use json;

/// Handler for ATLAS requests.
///
/// Just like the `Cameras` multi-route handler, this structure does not implement `Handler`
/// itself. Rather, its method(s) are passed via closures into the router.
#[derive(Clone, Debug)]
pub struct Atlas {
    config: Config,
}

impl From<Config> for Atlas {
    fn from(config: Config) -> Atlas {
        Atlas { config: config }
    }
}

impl Atlas {
    /// Returns a full status report for the ATLAS system.
    pub fn status(&self, _: &mut Request) -> IronResult<Response> {
        json::response(itry!(Status::new(&self.config)))
    }
}

#[cfg(test)]
mod tests {
    use {Api, Config};
    use atlas::config::EfoyCartridgeConfig;
    use iron::Headers;
    use iron_test::{request, response};
    use serde_json::{self, Value};

    #[test]
    fn status() {
        let mut config = Config::default();
        config.atlas.path = "../glacio/data".to_string();
        config.atlas.efoy.cartridges = vec![
            EfoyCartridgeConfig {
                name: "1.1".to_string(),
                capacity: 8.0,
            },
            EfoyCartridgeConfig {
                name: "1.2".to_string(),
                capacity: 8.0,
            },
        ];
        let api = Api::new(config).unwrap();
        let response = request::get("http://localhost:3000/atlas/status", Headers::new(), &api)
            .unwrap();
        let status: Value = serde_json::from_str(&response::extract_body_to_string(response))
            .unwrap();
        assert_eq!(
            "2017-08-25T15:01:06+00:00",
            status["last_heartbeat_received"]
        );
        assert_eq!("2017-08-25T12:02:08+00:00", status["last_scan"]["start"]);
        assert_eq!("2017-08-25T12:41:42+00:00", status["last_scan"]["end"]);
        assert_eq!(1, status["batteries"][0]["id"]);
        assert_eq!(85.461, status["batteries"][0]["state_of_charge"]);
        assert_eq!(2, status["batteries"][1]["id"]);
        assert_eq!(86.604, status["batteries"][1]["state_of_charge"]);

        assert_eq!(1, status["efoys"][0]["id"]);
        assert_eq!("auto off", status["efoys"][0]["state"]);
        assert_eq!("1.1", status["efoys"][0]["active_cartridge"]);
        assert_eq!(7.392, status["efoys"][0]["active_cartridge_consumed"]);
        assert_eq!(26.86, status["efoys"][0]["voltage"]);
        assert_eq!(-0.03, status["efoys"][0]["current"]);

        assert_eq!(2, status["efoys"][1]["id"]);
        assert_eq!("auto off", status["efoys"][1]["state"]);
        assert_eq!("1.2", status["efoys"][1]["active_cartridge"]);
        assert_eq!(0.049, status["efoys"][1]["active_cartridge_consumed"]);
        assert_eq!(26.86, status["efoys"][1]["voltage"]);
        assert_eq!(-0.04, status["efoys"][1]["current"]);

        assert_eq!(
            "2017-08-01T00:00:55+00:00",
            status["timeseries"]["datetimes"][0]
        );
        assert_eq!(94.208, status["timeseries"]["states_of_charge"]["1"][0]);
        assert_eq!(-0.03, status["timeseries"]["efoy_current"]["1"][0]);
        assert_eq!(26.63, status["timeseries"]["efoy_voltage"]["1"][0]);
        assert_eq!(
            76.61875,
            status["timeseries"]["efoy_fuel_percentage"]["1"][0]
        );
        assert_eq!("auto off", status["timeseries"]["efoy_state"]["1"][0]);
        assert_eq!(true, status["timeseries"]["is_riegl_switch_on"][0]);
    }
}
