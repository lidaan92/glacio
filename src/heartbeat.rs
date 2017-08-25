use {Error, Result};
use sbd::mo::Message;
use sbd::storage::{FilesystemStorage, Storage};
use std::path::Path;

/// Returns a `ReadSbd`, which can iterate over the heartbeats in an sbd storage.
///
/// # Examples
///
/// ```
/// # use glacio::heartbeat;
/// for result in heartbeat::read_sbd("data", "300234063556840").unwrap() {
///     let heartbeat = result.unwrap();
///     println!("{:?}", heartbeat);
/// }
/// ```
pub fn read_sbd<P: AsRef<Path>>(path: P, imei: &str) -> Result<ReadSbd> {
    FilesystemStorage::open(path)
        .and_then(|storage| storage.messages_from_imei(imei))
        .map(|messages| ReadSbd { messages: messages })
        .map_err(Error::from)
}

/// An iterator over heartbeats in an sbd storage.
///
/// The iterator type is a `Result<Heartbeat>`, because we can fail in the middle of a stream of
/// heartbeats.
#[derive(Debug)]
pub struct ReadSbd {
    messages: Vec<Message>,
}

/// A heartbeat from the ATLAS system.
///
/// These heartbeats are transmitted via Iridium SBD. Because of the SBD message length
/// restriction, heartbeats may come in one or more messages, and might have to be pieced together.
/// There are multiple version of the heartbeat messages, since Pete changes the format each time
/// he visits ATLAS.
#[derive(Clone, Copy, Debug)]
pub struct Heartbeat;

impl Iterator for ReadSbd {
    type Item = Result<Heartbeat>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeats() {
        let read_sbd = read_sbd("data", "300234063556840").unwrap();
        let heartbeats = read_sbd.collect::<Vec<Result<Heartbeat>>>();
        assert_eq!(2, heartbeats.len());
        assert!(heartbeats.iter().all(|result| result.is_ok()));
    }
}
