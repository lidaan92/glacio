use {Error, Result};
use regex::Regex;
use sbd::mo::Message;
use sbd::storage::{FilesystemStorage, Storage};
use std::path::Path;
use std::str::FromStr;
use std::vec::IntoIter;

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
        .map(|messages| ReadSbd { iter: messages.into_iter() })
        .map_err(Error::from)
}

/// An iterator over heartbeats in an sbd storage.
///
/// The iterator type is a `Result<Heartbeat>`, because we can fail in the middle of a stream of
/// heartbeats.
#[derive(Debug)]
pub struct ReadSbd {
    iter: IntoIter<Message>,
}

/// A heartbeat from the ATLAS system.
///
/// These heartbeats are transmitted via Iridium SBD. Because of the SBD message length
/// restriction, heartbeats may come in one or more messages, and might have to be pieced together.
/// There are multiple version of the heartbeat messages, since Pete changes the format each time
/// he visits ATLAS.
#[derive(Clone, Copy, Debug)]
pub struct Heartbeat;

/// A Sutron interleaved message.
///
/// Sutron will break up large messages into parts, using a header to define the number of bytes in
/// the complete message.
#[derive(Debug)]
struct InterleavedMessage {
    complete: bool,
    id: Option<String>,
    total_bytes: usize,
    data: String,
}

impl Iterator for ReadSbd {
    type Item = Result<Heartbeat>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut message = InterleavedMessage::new();
        while let Some(sbd_message) = self.iter.next() {
            match sbd_message.payload_str() {
                Ok(payload) => {
                    match message.add(payload) {
                        Ok(()) => if message.is_complete() {
                            return Some(message.data().parse());
                        },
                        Err(err) => return Some(Err(err)),
                    }
                }
                Err(err) => return Some(Err(err.into())),
            }
        }
        None
    }
}

impl FromStr for Heartbeat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Heartbeat> {
        Ok(Heartbeat)
    }
}

impl InterleavedMessage {
    fn new() -> InterleavedMessage {
        InterleavedMessage {
            id: None,
            total_bytes: 0,
            complete: false,
            data: String::new(),
        }
    }

    fn data(&self) -> &str {
        &self.data
    }

    fn add(&mut self, data: &str) -> Result<()> {
        if self.complete {
            return Err(Error::InterleavedMessage(format!("Trying to add data to an already-complete message: {}",
                                                         data)));
        }
        match &data[0..1] {
            "0" => {
                self.data.push_str(&data[1..]);
                self.complete = true;
                Ok(())
            }
            "1" => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"(?sx)^1,
                                                        (?P<id>\d+),
                                                        (?P<start_byte>\d+)
                                                        (,(?P<total_bytes>\d+))?:(?P<data>.*)$").unwrap();
                }
                if let Some(captures) = RE.captures(data) {
                    let id = captures.name("id").unwrap();
                    let start_byte = captures.name("start_byte").unwrap();
                    if start_byte == "0" {
                        self.id = Some(id.to_string());
                        if let Some(total_bytes) = captures.name("total_bytes") {
                            self.total_bytes = total_bytes.parse()?;
                        } else {
                            return Err(Error::InterleavedMessage("No total_bytes field for the first part of message".to_string()));
                        }
                    } else if let Some(ref previous_id) = self.id {
                        if id != previous_id {
                            return Err(Error::InterleavedMessage(format!("Ids don't match: {} <> {}",
                                                                         id,
                                                                         previous_id)));
                        }
                    } else {
                        return Err(Error::InterleavedMessage("Picking up message in the middle"
                                                                 .to_string()));
                    }
                    self.data.push_str(captures.name("data").unwrap());
                    if self.data.len() == self.total_bytes {
                        self.complete = true;
                    } else if self.data.len() > self.total_bytes {
                        return Err(Error::InterleavedMessage(format!("Too many bytes in data: {} (expected {})",
                                                                     self.data.len(),
                                                                     self.total_bytes)));
                    }
                    Ok(())
                } else {
                    return Err(Error::InterleavedMessage(format!("Message does not match regex: {}",
                                                                 data)));
                }
            }
            c @ _ => {
                return Err(Error::InterleavedMessage(format!("Unrecognized packet type: {}", c)))
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
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
