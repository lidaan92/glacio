use {Result, heartbeat};
use iron::Request;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    path: PathBuf,
    imei: String,
}

#[derive(Debug, Serialize)]
pub struct Status {
    last_heartbeat_received: String,
    soc1: f32,
    soc2: f32,
}

impl Config {
    pub fn status(&self, request: &Request) -> Result<Status> {
        let mut heartbeats = heartbeat::read_sbd(&self.path, &self.imei)
            .map(|read_sbd| read_sbd.filter_map(|result| result.ok()).collect::<Vec<_>>())
            .unwrap();
        heartbeats.sort_by(|a, b| b.cmp(&a));
        assert!(heartbeats.len() > 0);
        let latest = heartbeats[0];
        Ok(Status {
               last_heartbeat_received: latest.datetime.to_rfc3339(),
               soc1: latest.soc1,
               soc2: latest.soc2,
           })
    }
}
