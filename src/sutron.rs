use {Error, Result};
use regex::Regex;
use std::str::FromStr;

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
            Packet::SelfTimedExtended { data, .. } => data,
        }
    }
}

impl FromStr for Packet {
    type Err = Error;
    fn from_str(s: &str) -> Result<Packet> {
        match &s[0..1] {
            "0" => Ok(Packet::SelfTimed(s[1..].to_string())),
            "1" => {
                lazy_static! {
                    static ref RE: Regex = Regex::new(r"(?sx)^1,
                                                        (?P<id>\d+),
                                                        (?P<start_byte>\d+)
                                                        (,(?P<total_bytes>\d+))?:(?P<data>.*)$").unwrap();
                }
                if let Some(captures) = RE.captures(s) {
                    Ok(Packet::SelfTimedExtended {
                           id: captures.name("id")
                               .unwrap()
                               .parse()?,
                           start_byte: captures.name("start_byte")
                               .unwrap()
                               .parse()?,
                           total_bytes: captures.name("total_bytes")
                               .map_or(Ok(None), |s| s.parse().map(Some))?,
                           data: captures.name("data").unwrap().to_string(),
                       })
                } else {
                    Err(Error::InterleavedMessage(format!("Invalid self timed extended packet: {}",
                                                          s)))
                }
            }
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
}
