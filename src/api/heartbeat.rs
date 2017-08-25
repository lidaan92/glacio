use {Result, heartbeat};
use iron::Request;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    path: PathBuf,
    imei: String,
}

#[derive(Debug, Serialize)]
pub struct Status;

impl Config {
    pub fn status(&self, request: &Request) -> Result<Status> {
        let mut heartbeats = heartbeat::read_sbd(&self.path, &self.imei)
            .map(|read_sbd| read_sbd.filter_map(|result| result.ok()).collect::<Vec<_>>())
            .unwrap();
        heartbeats.sort_by(|a, b| b.cmp(&a));
        Ok(Status)
    }
}
