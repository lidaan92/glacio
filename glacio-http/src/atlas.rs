use {Error, Result};
use glacio::atlas::{Heartbeat, ReadSbd, SbdSource, efoy};
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
    /// The fuel level in 1.1.
    pub fuel_1_1: f32,
    /// The fuel level in 1.2.
    pub fuel_1_2: f32,
    /// The fuel level in 2.1.
    pub fuel_2_1: f32,
    /// The fuel level in 2.2.
    pub fuel_2_2: f32,
}

/// A record of the power status at a given time.
#[derive(Debug, Default, Serialize)]
pub struct PowerHistory {
    /// The date and time of the records.
    pub datetime: Vec<String>,
    /// The state of charge of battery 1.
    pub state_of_charge_1: Vec<f32>,
    /// The state of charge of battery 2.
    pub state_of_charge_2: Vec<f32>,
    /// The output current of efoy 1.
    pub efoy_1_current: Vec<f32>,
    /// The fuel level in EFOY 1, as a percentage.
    pub efoy_1_fuel: Vec<f32>,
    /// The output current of efoy 2.
    pub efoy_2_current: Vec<f32>,
    /// The fuel level in EFOY 1, as a percentage.
    pub efoy_2_fuel: Vec<f32>,
}

impl Config {
    pub fn status(&self, _: &Request) -> Result<Status> {
        let mut heartbeats = self.heartbeats()?;
        heartbeats.sort();
        let mut efoy1 = efoy::Efoy::new();
        let mut efoy2 = efoy::Efoy::new();
        for heartbeat in &heartbeats {
            efoy1.process(&heartbeat.efoy1)?;
            efoy2.process(&heartbeat.efoy2)?;
        }
        let latest = heartbeats.into_iter().last().unwrap();
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
               efoys: vec![EfoyStatus::new(1, &latest.efoy1, &efoy1),
                           EfoyStatus::new(2, &latest.efoy2, &efoy2)],
           })
    }

    pub fn power_history(&self, _: &Request) -> Result<PowerHistory> {
        let mut heartbeats = self.heartbeats()?;
        heartbeats.sort();
        let mut power_history: PowerHistory = Default::default();
        let mut efoy1 = efoy::Efoy::new();
        let mut efoy2 = efoy::Efoy::new();
        for heartbeat in heartbeats {
            efoy1.process(&heartbeat.efoy1)?;
            efoy2.process(&heartbeat.efoy2)?;
            power_history.datetime.push(heartbeat.datetime.to_rfc3339());
            power_history.state_of_charge_1.push(heartbeat.soc1);
            power_history.state_of_charge_2.push(heartbeat.soc1);
            power_history.efoy_1_current.push(heartbeat.efoy1.current);
            power_history.efoy_1_fuel.push(efoy1.total_fuel_percentage());
            power_history.efoy_2_current.push(heartbeat.efoy2.current);
            power_history.efoy_2_fuel.push(efoy2.total_fuel_percentage());
        }
        Ok(power_history)
    }

    pub fn heartbeats(&self) -> Result<Vec<Heartbeat>> {
        let heartbeats: Vec<Heartbeat> = self.read_sbd()?
            .flat_map(|result| result.ok())
            .collect();
        if heartbeats.is_empty() {
            return Err(Error::Config(format!("No heartbeats found under path {}",
                                             self.path.display())));
        }
        Ok(heartbeats)
    }

    pub fn read_sbd(&self) -> Result<ReadSbd> {
        SbdSource::new(&self.path)
            .imeis(&[&self.imei])
            .versions(&self.versions)
            .iter()
            .map_err(Error::from)
    }
}

impl EfoyStatus {
    fn new(id: u8, efoy_heartbeat: &efoy::Heartbeat, efoy: &efoy::Efoy) -> EfoyStatus {
        EfoyStatus {
            id: id,
            state: efoy_heartbeat.state.into(),
            cartridge: efoy_heartbeat.cartridge.clone(),
            consumed: efoy_heartbeat.consumed,
            voltage: efoy_heartbeat.voltage,
            current: efoy_heartbeat.current,
            fuel_1_1: efoy.fuel_percentage("1.1").unwrap(),
            fuel_1_2: efoy.fuel_percentage("1.2").unwrap(),
            fuel_2_1: efoy.fuel_percentage("2.1").unwrap(),
            fuel_2_2: efoy.fuel_percentage("2.2").unwrap(),
        }
    }
}
