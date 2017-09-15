use Result;
use atlas::Config;
use glacio::atlas::{Efoy, Heartbeat, efoy};
use std::collections::BTreeMap;

/// An ATLAS status report.
#[derive(Debug, Serialize)]
pub struct Status {
    /// The date and time that the last heartbeat was received.
    pub last_heartbeat_received: String,
    /// A list of battery status information.
    pub batteries: Vec<BatteryStatus>,
    /// A list of efoy status information.
    pub efoys: Vec<EfoyStatus>,
    /// Information about the last scan.
    pub last_scan: LastScan,
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
    pub states_of_charge: BTreeMap<u8, Vec<f32>>,
    /// A map from efoy id to current level.
    pub efoy_current: BTreeMap<u8, Vec<f32>>,
    /// A map from efoy id to voltage level.
    pub efoy_voltage: BTreeMap<u8, Vec<f32>>,
    /// A map from efoy id to fuel level.
    pub efoy_fuel_percentage: BTreeMap<u8, Vec<f32>>,
    #[serde(skip)]
    efoys: BTreeMap<u8, Efoy>,
}

/// The last scan.
#[derive(Debug, Serialize)]
pub struct LastScan {
    start: String,
    end: Option<String>,
}

impl Status {
    /// Creates a new status from a configuration and a request.
    pub fn new(config: &Config) -> Result<Status> {
        let mut heartbeats = config.heartbeats()?;
        heartbeats.sort();
        let mut timeseries = Timeseries::new(config, &heartbeats[0])?;
        for heartbeat in &heartbeats {
            timeseries.process(&heartbeat)?;
        }
        let heartbeat = heartbeats.pop().unwrap();
        let batteries = heartbeat.batteries
            .iter()
            .map(|(&i, battery)| BatteryStatus::new(i, battery.state_of_charge))
            .collect();
        Ok(Status {
               last_heartbeat_received: heartbeat.datetime.to_rfc3339(),
               batteries: batteries,
               efoys: timeseries.efoys(&heartbeat),
               timeseries: timeseries,
               are_riegl_systems_on: heartbeat.are_riegl_systems_on,
               last_scan: LastScan::new(&heartbeat),
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
    fn new(config: &Config, heartbeat: &Heartbeat) -> Result<Timeseries> {
        let states_of_charge = heartbeat.batteries
            .keys()
            .map(|&i| (i, Vec::new()))
            .collect();
        let mut efoy_current = BTreeMap::new();
        let mut efoy_fuel_percentage = BTreeMap::new();
        let mut efoy_voltage = BTreeMap::new();
        let mut efoys = BTreeMap::new();
        for &i in heartbeat.efoys.keys() {
            efoy_current.insert(i, Vec::new());
            efoy_fuel_percentage.insert(i, Vec::new());
            efoy_voltage.insert(i, Vec::new());
            efoys.insert(i, config.efoy()?);
        }

        Ok(Timeseries {
               datetimes: Vec::new(),
               states_of_charge: states_of_charge,
               efoy_current: efoy_current,
               efoy_fuel_percentage: efoy_fuel_percentage,
               efoy_voltage: efoy_voltage,
               efoys: efoys,
           })
    }

    fn process(&mut self, heartbeat: &Heartbeat) -> Result<()> {
        self.datetimes.push(heartbeat.datetime.to_rfc3339());
        for (i, battery) in &heartbeat.batteries {
            self.states_of_charge
                .get_mut(i)
                .unwrap()
                .push(battery.state_of_charge);
        }
        for (i, heartbeat) in &heartbeat.efoys {
            self.efoy_current
                .get_mut(i)
                .unwrap()
                .push(heartbeat.current);
            self.efoy_voltage
                .get_mut(i)
                .unwrap()
                .push(heartbeat.voltage);
            let mut efoy = self.efoys.get_mut(i).unwrap();
            efoy.process(heartbeat)?;
            self.efoy_fuel_percentage
                .get_mut(i)
                .unwrap()
                .push(efoy.total_fuel_percentage());
        }
        Ok(())
    }

    fn efoys(&self, heartbeat: &Heartbeat) -> Vec<EfoyStatus> {
        self.efoys
            .iter()
            .map(|(&i, efoy)| EfoyStatus::new(i, efoy, &heartbeat.efoys[&i]))
            .collect()
    }
}

impl LastScan {
    fn new(heartbeat: &Heartbeat) -> LastScan {
        let start = heartbeat.scan_start;
        let end = heartbeat.scan_stop.datetime;
        LastScan {
            start: start.to_rfc3339(),
            end: if start < end {
                Some(end.to_rfc3339())
            } else {
                None
            },
        }
    }
}
