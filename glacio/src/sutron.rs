use {Error, Result};
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use std::str::FromStr;

lazy_static! {
    static ref SELF_TIMED_EXTENDED_REGEX: Regex = Regex::new(r"(?sx)^
        1,
        (?P<id>\d+),
        (?P<start_byte>\d+)
        (,(?P<total_bytes>\d+))?:(?P<data>.*)
        $").unwrap();
}
const DATETIME_FORMAT: &'static str = "%m/%d/%y %H:%M:%S";

#[derive(Clone, Debug)]
pub enum Message {
    Unstarted,
    Incomplete {
        id: u8,
        total_bytes: usize,
        data: String,
    },
    Complete(String),
}

#[derive(Clone, Debug)]
pub enum Packet {
    SelfTimed(String),
    SelfTimedExtended {
        id: u8,
        start_byte: usize,
        total_bytes: Option<usize>,
        data: String,
    },
    ForcedTransmission(String),
    ForcedTransmissionExtended(String),
}

pub fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    Utc.datetime_from_str(s, DATETIME_FORMAT).map_err(Error::from)
}

impl From<Message> for String {
    fn from(message: Message) -> String {
        match message {
            Message::Unstarted => String::new(),
            Message::Incomplete { data, .. } |
            Message::Complete(data) => data,
        }
    }
}

impl Default for Message {
    fn default() -> Message {
        Message::Unstarted
    }
}

impl Message {
    pub fn new() -> Message {
        Default::default()
    }

    pub fn push(self, payload: &str) -> Result<Message> {
        match (self, payload.parse::<Packet>()?) {
            (Message::Unstarted, Packet::SelfTimed(data)) => {
                Ok(Message::Complete(data.to_string()))
            }
            (Message::Unstarted,
             Packet::SelfTimedExtended { id, start_byte, total_bytes, data }) => {
                if start_byte != 0 {
                    Err(Error::InterleavedMessage("Start byte is not zero, cannot start packet"
                                                      .to_string()))
                } else if let Some(total_bytes) = total_bytes {
                    Ok(Message::Incomplete {
                           id: id,
                           total_bytes: total_bytes,
                           data: data,
                       })
                } else {
                    Err(Error::InterleavedMessage("Total bytes must be specified for start packet"
                                                      .to_string()))
                }
            }
            (Message::Incomplete { .. }, Packet::SelfTimed(_)) => {
                Err(Error::InterleavedMessage("Cannot add non-extended packet to incomplete message"
                                                  .to_string()))
            }
            (Message::Incomplete { id, total_bytes, data },
             Packet::SelfTimedExtended { id: packet_id, start_byte, data: packet_data, .. }) => {
                if packet_id != id {
                    Err(Error::InterleavedMessage(format!("Packet id ({}) does not match message id ({})",
                                                          packet_id,
                                                          id)))
                } else if start_byte != data.len() {
                    Err(Error::InterleavedMessage(format!("Recieved {} bytes, but start byte is {}",
                                                          data.len(),
                                                          start_byte)))
                } else {
                    let data = data + &packet_data;
                    if data.len() == total_bytes {
                        Ok(Message::Complete(data))
                    } else {
                        Ok(Message::Incomplete {
                               id: id,
                               total_bytes: total_bytes,
                               data: data,
                           })
                    }
                }
            }
            (Message::Complete(_), _) => {
                Err(Error::InterleavedMessage("Message is already complete, cannot add more"
                                                  .to_string()))
            }
            (_, Packet::ForcedTransmission(message)) => {
                Err(Error::InterleavedMessage(format!("Forced transmission: {}", message)))
            }
            (_, Packet::ForcedTransmissionExtended(message)) => {
                Err(Error::InterleavedMessage(format!("Forced transmission extended: {}", message)))
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        match *self {
            Message::Unstarted |
            Message::Incomplete { .. } => false,
            Message::Complete(_) => true,
        }
    }
}

impl From<Packet> for String {
    fn from(packet: Packet) -> String {
        match packet {
            Packet::SelfTimed(data) |
            Packet::SelfTimedExtended { data, .. } |
            Packet::ForcedTransmission(data) |
            Packet::ForcedTransmissionExtended(data) => data,
        }
    }
}

impl FromStr for Packet {
    type Err = Error;
    fn from_str(s: &str) -> Result<Packet> {
        use utils;

        match &s[0..1] {
            "0" => Ok(Packet::SelfTimed(s[1..].to_string())),
            "1" => {
                if let Some(ref captures) = SELF_TIMED_EXTENDED_REGEX.captures(s) {
                    Ok(Packet::SelfTimedExtended {
                           id: utils::parse_capture(captures, "id")?,
                           start_byte: utils::parse_capture(captures, "start_byte")?,
                           total_bytes: captures.name("total_bytes")
                               .map_or(Ok(None), |s| s.as_str().parse().map(Some))?,
                           data: captures.name("data")
                               .unwrap()
                               .as_str()
                               .to_string(),
                       })
                } else {
                    Err(Error::InterleavedMessage(format!("Invalid self timed extended packet: {}",
                                                          s)))
                }
            }
            "8" => Ok(Packet::ForcedTransmission(s[1..].to_string())),
            "9" => Ok(Packet::ForcedTransmissionExtended(s[1..].to_string())),
            c => Err(Error::InterleavedMessage(format!("Unsupported packet type: {}", c))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SELF_TIMED: &'static str = "0ATHB03313";
    const SELF_TIMED_EXTENDED_0: &'static str = include_str!("../data/170801_000055.txt");
    const SELF_TIMED_EXTENDED_1: &'static str = include_str!("../data/170801_000155.txt");
    const FORCED_TRANSMISSION: &'static str = include_str!("../data/160719_193136.txt");

    #[test]
    fn message_add_self_timed() {
        let mut message = Message::new();
        message = message.push(SELF_TIMED).unwrap();
        assert!(message.is_complete());
        assert_eq!("ATHB03313", String::from(message));
    }

    #[test]
    fn message_add_self_timed_extended() {
        let mut message = Message::new();
        message = message.push(SELF_TIMED_EXTENDED_0).unwrap();
        assert!(!message.is_complete());
        message = message.push(SELF_TIMED_EXTENDED_1).unwrap();
        assert!(message.is_complete());
        assert_eq!(354, String::from(message.clone()).len());
        assert!(message.push(SELF_TIMED_EXTENDED_1).is_err());
    }

    #[test]
    fn forced_transmission() {
        match FORCED_TRANSMISSION.parse::<Packet>().unwrap() {
            Packet::ForcedTransmission(msg) => assert_eq!("test", msg),
            _ => panic!("Forced transmission was not recognized as such"),
        }
    }
}
