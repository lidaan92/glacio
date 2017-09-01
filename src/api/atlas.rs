use {Error, Result};
use atlas::{Heartbeat, SbdSource};
use iron::Request;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    path: PathBuf,
    imei: String,
    versions: Vec<u8>,
}

/// The status of the ATLAS system.
///
/// Some of this information is taken directly from the last heartbeat, and some of it is
/// calculated from the ATLAS state machine.
#[derive(Debug, Serialize)]
pub struct Status {
    /// The date and time the last heartbeat was received.
    pub last_heartbeat_received: String,
    /// All batteries hooked into the system.
    pub batteries: Vec<BatteryStatus>,
}

/// The status of an ATLAS battery.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct BatteryStatus {
    /// The identification number of the battery.
    pub id: u8,
    /// The state of charge of the battery, as a percentage.
    pub state_of_charge: f32,
}

/// A record of the power status at a given time.
#[derive(Debug, Serialize)]
pub struct PowerHistory {
    /// The date and time of the records.
    pub datetime: Vec<String>,
    /// The state of charge of battery 1.
    pub state_of_charge_1: Vec<f32>,
    /// The state of charge of battery 2.
    pub state_of_charge_2: Vec<f32>,
}

impl Config {
    pub fn status(&self, _: &Request) -> Result<Status> {
        let mut heartbeats = self.heartbeats()?;
        heartbeats.sort_by(|a, b| b.cmp(a));
        let latest = heartbeats[0];
        Ok(Status {
               last_heartbeat_received: latest.datetime.to_rfc3339(),
               batteries: vec![BatteryStatus {
                                   id: 1,
                                   state_of_charge: latest.soc1,
                               },
                               BatteryStatus {
                                   id: 2,
                                   state_of_charge: latest.soc2,
                               }],
           })
    }

    pub fn power_history(&self, _: &Request) -> Result<PowerHistory> {
        let mut heartbeats = self.heartbeats()?;
        heartbeats.sort();
        let mut datetime = Vec::new();
        let mut state_of_charge_1 = Vec::new();
        let mut state_of_charge_2 = Vec::new();
        for heartbeat in heartbeats {
            datetime.push(heartbeat.datetime.to_rfc3339());
            state_of_charge_1.push(heartbeat.soc1);
            state_of_charge_2.push(heartbeat.soc1);
        }
        Ok(PowerHistory {
               datetime: datetime,
               state_of_charge_1: state_of_charge_1,
               state_of_charge_2: state_of_charge_2,
           })
    }

    fn heartbeats(&self) -> Result<Vec<Heartbeat>> {
        let heartbeats: Vec<Heartbeat> = SbdSource::new(&self.path)
            .imeis(&[&self.imei])
            .versions(&self.versions)
            .iter()?
            .flat_map(|result| result.ok())
            .collect();
        if heartbeats.is_empty() {
            return Err(Error::ApiConfig("no heartbeats found".to_string()));
        }
        Ok(heartbeats)
    }
}
