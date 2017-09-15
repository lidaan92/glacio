use Result;
use atlas::Config;
use glacio::atlas::{Efoy, Heartbeat, efoy};
use std::collections::HashMap;

/// An ATLAS status report.
#[derive(Debug, Serialize)]
pub struct Status {
    /// The date and time that the last heartbeat was received.
    pub last_heartbeat_received: String,
    /// A list of battery status information.
    pub batteries: Vec<BatteryStatus>,
    /// A list of efoy status information.
    pub efoys: Vec<EfoyStatus>,
    /// Timeseries information, used to provide historical context.
    pub timeseries: Timeseries,
    /// Are the Riegl systems powered?
    pub are_riegl_systems_on: bool,
}

/// The status of one of the battery systems.
#[derive(Debug, Serialize)]
pub struct BatteryStatus {
    /// The battery id number.
    pub id: u8,
    /// The state of charge of the battery system, as a percentage between zero and 100.
    pub state_of_charge: f32,
}

/// The status of one of the EFOY fuel cell systems.
#[derive(Debug, Serialize)]
pub struct EfoyStatus {
    /// The EFOY id number.
    pub id: u8,
    /// The state of the efoy system, such as "auto on".
    pub state: String,
    /// The name of the active cartridge.
    pub active_cartridge: String,
    /// The amount consumed out of the active cartridge.
    pub active_cartridge_consumed: f32,
    /// The voltage level of the EFOY's sense/power lines.
    pub voltage: f32,
    /// The current level of the EFOY's sense/power lines.
    pub current: f32,
    /// A list of EFOY fuel cartridge status reports.
    pub cartridges: Vec<CartridgeStatus>,
}

/// The status of an EFOY cartridge.
#[derive(Debug, Serialize)]
pub struct CartridgeStatus {
    /// The name of the cartridge.
    pub name: String,
    /// The amont of fuel remaining in the cartridge, as a percentage between zero and 100.
    pub fuel_percentage: f32,
}

/// A timeseries of information about the ATLAS system.
///
/// We don't want to duplicate keys when pushing JSON, so this object has many vector members, instead of being
/// included in a vector itself.
#[derive(Debug, Serialize)]
pub struct Timeseries {
    /// The list of dates and times for this timeseries.
    pub datetimes: Vec<String>,
    /// A map from battery id to a list of states of charge for that battery.
    pub states_of_charge: HashMap<u8, Vec<f32>>,
    /// A map from efoy id to current level.
    pub efoy_current: HashMap<u8, Vec<f32>>,
    /// A map from efoy id to voltage level.
    pub efoy_voltage: HashMap<u8, Vec<f32>>,
    /// A map from efoy id to fuel level.
    pub efoy_fuel_percentage: HashMap<u8, Vec<f32>>,
}

impl Status {
    /// Creates a new status from a configuration and a request.
    pub fn new(config: &Config) -> Result<Status> {
        let mut heartbeats = config.heartbeats()?;
        heartbeats.sort();
        let mut efoy1 = config.efoy()?;
        let mut efoy2 = config.efoy()?;
        let mut timeseries = Timeseries::new();
        for heartbeat in &heartbeats {
            efoy1.process(&heartbeat.efoy1)?;
            efoy2.process(&heartbeat.efoy2)?;
            timeseries.process(&heartbeat, &efoy1, &efoy2);
        }
        let heartbeat = heartbeats.pop().unwrap();
        let batteries = vec![BatteryStatus::new(1, heartbeat.soc1),
                             BatteryStatus::new(2, heartbeat.soc2)];
        let efoys = vec![EfoyStatus::new(1, &efoy1, &heartbeat.efoy1),
                         EfoyStatus::new(2, &efoy2, &heartbeat.efoy2)];
        Ok(Status {
               last_heartbeat_received: heartbeat.datetime.to_rfc3339(),
               batteries: batteries,
               efoys: efoys,
               timeseries: timeseries,
               are_riegl_systems_on: heartbeat.are_riegl_systems_on,
           })
    }
}

impl BatteryStatus {
    fn new(id: u8, state_of_charge: f32) -> BatteryStatus {
        BatteryStatus {
            id: id,
            state_of_charge: state_of_charge,
        }
    }
}

impl EfoyStatus {
    fn new(id: u8, efoy: &Efoy, heartbeat: &efoy::Heartbeat) -> EfoyStatus {
        EfoyStatus {
            id: id,
            state: String::from(heartbeat.state),
            active_cartridge: heartbeat.cartridge.to_string(),
            active_cartridge_consumed: heartbeat.consumed,
            voltage: heartbeat.voltage,
            current: heartbeat.current,
            cartridges: efoy.iter()
                .map(|cartridge| {
                         CartridgeStatus {
                             name: cartridge.name().to_string(),
                             fuel_percentage: cartridge.fuel_percentage(),
                         }
                     })
                .collect(),
        }
    }
}

impl Timeseries {
    fn new() -> Timeseries {
        let mut states_of_charge = HashMap::new();
        let mut efoy_current = HashMap::new();
        let mut efoy_fuel_percentage = HashMap::new();
        let mut efoy_voltage = HashMap::new();
        for id in 0..2 {
            states_of_charge.insert(id + 1, Vec::new());
            efoy_current.insert(id + 1, Vec::new());
            efoy_fuel_percentage.insert(id + 1, Vec::new());
            efoy_voltage.insert(id + 1, Vec::new());
        }
        Timeseries {
            datetimes: Vec::new(),
            states_of_charge: states_of_charge,
            efoy_current: efoy_current,
            efoy_fuel_percentage: efoy_fuel_percentage,
            efoy_voltage: efoy_voltage,
        }
    }

    fn process(&mut self, heartbeat: &Heartbeat, efoy1: &Efoy, efoy2: &Efoy) {
        self.datetimes.push(heartbeat.datetime.to_rfc3339());
        self.states_of_charge
            .get_mut(&1)
            .unwrap()
            .push(heartbeat.soc1);
        self.states_of_charge
            .get_mut(&2)
            .unwrap()
            .push(heartbeat.soc2);
        self.efoy_current
            .get_mut(&1)
            .unwrap()
            .push(heartbeat.efoy1.current);
        self.efoy_current
            .get_mut(&2)
            .unwrap()
            .push(heartbeat.efoy2.current);
        self.efoy_voltage
            .get_mut(&1)
            .unwrap()
            .push(heartbeat.efoy1.current);
        self.efoy_voltage
            .get_mut(&2)
            .unwrap()
            .push(heartbeat.efoy2.current);
        self.efoy_fuel_percentage
            .get_mut(&1)
            .unwrap()
            .push(efoy1.total_fuel_percentage());
        self.efoy_fuel_percentage
            .get_mut(&2)
            .unwrap()
            .push(efoy2.total_fuel_percentage());
    }
}
