use {Result, atlas};
use iron::Request;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    path: PathBuf,
    imei: String,
}

/// The status of the ATLAS system.
///
/// Some of this information is taken directly from the last heartbeat, and some of it is
/// calculated from the ATLAS state machine.
#[derive(Debug, Serialize)]
pub struct Status {
    /// The date and time the last heartbeat was received.
    pub last_heartbeat_received: String,
    /// The state of charge of battery 1.
    pub soc1: f32,
    /// The state of charge of battery 2.
    pub soc2: f32,
}

impl Config {
    pub fn status(&self, _: &Request) -> Result<Status> {
        // TODO we should have a more robust reader for sbd.
        let mut heartbeats = atlas::read_sbd(&self.path, &self.imei)
            .map(|read_sbd| read_sbd.filter_map(|result| result.ok()).collect::<Vec<_>>())
            .unwrap();
        heartbeats.sort_by(|a, b| b.cmp(a));
        // FIXME
        assert!(!heartbeats.is_empty());
        let latest = heartbeats[0];
        Ok(Status {
               last_heartbeat_received: latest.datetime.to_rfc3339(),
               soc1: latest.soc1,
               soc2: latest.soc2,
           })
    }
}
