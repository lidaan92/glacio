use {Error, Result};
use atlas::{Efoy, Heartbeat, ReadSbd, SbdSource};
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
    /// All the efoys connected into the system.
    pub efoys: Vec<EfoyStatus>,
}

/// The status of an ATLAS battery.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct BatteryStatus {
    /// The identification number of the battery.
    pub id: u8,
    /// The state of charge of the battery, as a percentage.
    pub state_of_charge: f32,
}

/// The status of an EFOY system.
#[derive(Clone, Debug, Serialize)]
pub struct EfoyStatus {
    /// The numeric id of this efoy system.
    pub id: u8,
    /// The state of the efoy system.
    pub state: String,
    /// The active cartridge.
    pub cartridge: String,
    /// The fuel consumed out of this cartridge.
    pub consumed: f32,
    /// The voltage level of this efoy.
    pub voltage: f32,
    /// The current level of this efoy.
    pub current: f32,
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
    /// Is EFOY 1 on?
    pub efoy_1_on: Vec<bool>,
    /// Is EFOY 2 on?
    pub efoy_2_on: Vec<bool>,
}

impl Config {
    pub fn status(&self, _: &Request) -> Result<Status> {
        let mut heartbeats = self.heartbeats()?;
        heartbeats.sort_by(|a, b| b.cmp(a));
        let latest = &heartbeats[0];
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
               efoys: vec![EfoyStatus::new(1, &latest.efoy1), EfoyStatus::new(2, &latest.efoy2)],
           })
    }

    pub fn power_history(&self, _: &Request) -> Result<PowerHistory> {
        let mut heartbeats = self.heartbeats()?;
        heartbeats.sort();
        let mut datetime = Vec::new();
        let mut state_of_charge_1 = Vec::new();
        let mut state_of_charge_2 = Vec::new();
        let mut efoy_1_on = Vec::new();
        let mut efoy_2_on = Vec::new();
        for heartbeat in heartbeats {
            datetime.push(heartbeat.datetime.to_rfc3339());
            state_of_charge_1.push(heartbeat.soc1);
            state_of_charge_2.push(heartbeat.soc1);
            efoy_1_on.push(heartbeat.efoy1.is_on());
            efoy_2_on.push(heartbeat.efoy2.is_on());
        }
        Ok(PowerHistory {
               datetime: datetime,
               state_of_charge_1: state_of_charge_1,
               state_of_charge_2: state_of_charge_2,
               efoy_1_on: efoy_1_on,
               efoy_2_on: efoy_2_on,
           })
    }

    pub fn heartbeats(&self) -> Result<Vec<Heartbeat>> {
        let heartbeats: Vec<Heartbeat> = self.read_sbd()?
            .flat_map(|result| result.ok())
            .collect();
        if heartbeats.is_empty() {
            return Err(Error::ApiConfig("no heartbeats found".to_string()));
        }
        Ok(heartbeats)
    }

    pub fn read_sbd(&self) -> Result<ReadSbd> {
        SbdSource::new(&self.path).imeis(&[&self.imei]).versions(&self.versions).iter()
    }
}

impl EfoyStatus {
    fn new(id: u8, efoy: &Efoy) -> EfoyStatus {
        EfoyStatus {
            id: id,
            state: efoy.state.into(),
            cartridge: efoy.cartridge.clone(),
            consumed: efoy.consumed,
            voltage: efoy.voltage,
            current: efoy.current,
        }
    }
}
