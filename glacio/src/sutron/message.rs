//! Sutron messages.
//!
//! Contains its own error enum, because there's a variety of errors that can arise while parsing
//! SBD messages sent by a Sutron system.

use regex::Regex;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::num::ParseIntError;
use std::result;
use std::str::FromStr;

lazy_static! {
    static ref SELF_TIMED_EXTENDED_REGEX: Regex = Regex::new(r"(?sx)^
        1,
        (?P<id>\d+),
        (?P<start_byte>\d+)
        (,(?P<total_bytes>\d+))?:(?P<data>.*)
        $").unwrap();
}

/// An interleaved SBD message.
///
/// In order to send a long text string over SBD, the Sutron data logger chops the message into
/// parts and sends it in several messages. To reconstruct the message, we have to read in one or
/// more packets of information.
#[derive(Clone, Debug)]
pub enum Message {
    /// An unstarted message. Add a packet to get it started.
    Unstarted,
    /// An incomplete message.
    Incomplete {
        /// The numeric id of all packets in this message.
        ///
        /// Future packets must match this id.
        id: u8,
        /// The total bytes in this message.
        ///
        /// As we add packets, we check to see if we hit/exceed the total bytes.
        total_bytes: usize,
        /// The message so far.
        data: String,
    },
    /// A complete message.
    Complete(String),
}

/// One SBD message's worth of information.
#[derive(Clone, Debug)]
pub enum Packet {
    /// A self-timed message that fits in one packet.
    ///
    /// Self-timed messages, in our case, contain regular information about a system, e.g. ATLAS
    /// heartbeats.
    SelfTimed(String),
    /// Part of an extended self-timed message.
    ///
    /// This message couldn't fit in one SBD message, so the Sutron data logger split it up over
    /// several extended packets.
    SelfTimedExtended {
        /// The id number of this extended message.
        id: u8,
        /// The start byte of this packet.
        ///
        /// Presumably, we've already read all the data up to this start byte.
        start_byte: usize,
        /// The total bytes in this message.
        ///
        /// Only present on the first packet of a message.
        total_bytes: Option<usize>,
        /// The payload of the packet.
        data: String,
    },
    /// A forced transmission.
    ///
    /// Someone (usually Pete) forced the data logger to send an SBD message, and that message fit
    /// in one SBD transmission.
    ///
    /// These are almost always test messages.
    ForcedTransmission(String),
    /// A forced transmission that had to be split up over multiple SBD transmissions.
    ForcedTransmissionExtended(String),
}

/// A custom error enum for reconstruction Sutron messages.
#[derive(Debug)]
pub enum Error {
    /// The number of bytes received doesn't match the start byte of the packet.
    ByteMismatch {
        /// The number of bytes received.
        received: usize,
        /// The start byte of the packet.
        start_byte: usize,
    },
    /// The packet id does not match the message id.
    IdMismatch {
        /// The packet id.
        packet: u8,
        /// The message id.
        message: u8,
    },
    /// The packet is in an invalid format.
    InvalidFormat(String),
    /// The message is complete, and cannot accept any more packets.
    MessageComplete,
    /// The initial packet is missing the total bytes field.
    MissingTotalBytes,
    /// A non-extended packet was added to an incomplete message.
    NonExtendedContinuationPacket,
    /// The start byte of the initial packet was not zero.
    NonzeroStartByte,
    /// Wrapper around `std::num::ParseIntError`.
    ParseInt(ParseIntError),
    /// The packet type is not supported.
    UnsupportedPacketType(String),
}

/// Custom result type for Sutron messages.
pub type Result<T> = result::Result<T, Error>;

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
    /// Creates a new, unstarted message.
    ///
    /// # Examples
    ///
    /// ```
    /// use glacio::sutron::Message;
    /// let message = Message::new();
    /// assert!(!message.is_complete());
    /// ```
    pub fn new() -> Message {
        Default::default()
    }

    /// Adds a packet, as a string, to this message.
    ///
    /// The message is consumed, and a new message is returned from the function.
    ///
    /// # Examples
    ///
    /// A packet starting with a "0" is self-timed:
    ///
    /// ```
    /// use glacio::sutron::Message;
    /// let mut message = Message::new().add("0A self timed message").unwrap();
    /// assert!(message.is_complete());
    /// assert_eq!("A self timed message", String::from(message));
    /// ```
    pub fn add(self, payload: &str) -> Result<Message> {
        match (self, payload.parse::<Packet>()?) {
            (Message::Unstarted, Packet::SelfTimed(data)) => {
                Ok(Message::Complete(data.to_string()))
            }
            (Message::Unstarted,
             Packet::SelfTimedExtended {
                 id,
                 start_byte,
                 total_bytes,
                 data,
             }) => {
                if start_byte != 0 {
                    Err(Error::NonzeroStartByte)
                } else if let Some(total_bytes) = total_bytes {
                    Ok(Message::Incomplete {
                        id: id,
                        total_bytes: total_bytes,
                        data: data,
                    })
                } else {
                    Err(Error::MissingTotalBytes)
                }
            }
            (Message::Incomplete { .. }, Packet::SelfTimed(_)) => {
                Err(Error::NonExtendedContinuationPacket)
            }
            (Message::Incomplete {
                 id,
                 total_bytes,
                 data,
             },
             Packet::SelfTimedExtended {
                 id: packet_id,
                 start_byte,
                 data: packet_data,
                 ..
             }) => {
                if packet_id != id {
                    Err(Error::IdMismatch {
                        packet: packet_id,
                        message: id,
                    })
                } else if start_byte != data.len() {
                    Err(Error::ByteMismatch {
                        received: data.len(),
                        start_byte: start_byte,
                    })
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
            (Message::Complete(_), _) => Err(Error::MessageComplete),
            (_, Packet::ForcedTransmission(message)) => Ok(Message::Complete(message)),
            (_, Packet::ForcedTransmissionExtended(message)) => Ok(Message::Complete(message)),
        }
    }

    /// Is this message complete?
    ///
    /// # Examples
    ///
    /// ```
    /// use glacio::sutron::Message;
    /// let mut message = Message::new();
    /// assert!(!message.is_complete());
    /// message = message.add("0this should complete it").unwrap();
    /// assert!(message.is_complete());
    /// ```
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
        match &s[0..1] {
            "0" => Ok(Packet::SelfTimed(s[1..].to_string())),
            "1" => {
                if let Some(ref captures) = SELF_TIMED_EXTENDED_REGEX.captures(s) {
                    Ok(Packet::SelfTimedExtended {
                        id: captures.name("id").unwrap().as_str().parse()?,
                        start_byte: captures.name("start_byte").unwrap().as_str().parse()?,
                        total_bytes: captures.name("total_bytes").map_or(Ok(None), |s| {
                            s.as_str().parse().map(Some)
                        })?,
                        data: captures.name("data").unwrap().as_str().to_string(),
                    })
                } else {
                    Err(Error::InvalidFormat(s.to_string()))
                }
            }
            "8" => Ok(Packet::ForcedTransmission(s[1..].to_string())),
            "9" => Ok(Packet::ForcedTransmissionExtended(s[1..].to_string())),
            c => Err(Error::UnsupportedPacketType(c.to_string())),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ByteMismatch { .. } => {
                "the number of bytes received does not match the start byte of the packet"
            }
            Error::IdMismatch { .. } => "the id of the packet and of the message do not match",
            Error::InvalidFormat(_) => {
                "the packet has an invalid format (does not match the packet regular expression"
            }
            Error::MessageComplete => "tried adding a packet to an already-completed message",
            Error::MissingTotalBytes => {
                "the total bytes field must be populated on an initial packet"
            }
            Error::NonExtendedContinuationPacket => {
                "cannot add a non-extended packet to a started (and incomplete) message"
            }
            Error::NonzeroStartByte => "the start byte for an initial packet must be zero",
            Error::ParseInt(ref err) => err.description(),
            Error::UnsupportedPacketType(_) => "this packet type is not supported",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ParseInt(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use std::error::Error as _Error;
        match *self {
            Error::ByteMismatch {
                received,
                start_byte,
            } => {
                write!(
                    f,
                    "received {} bytes, start byte is {}",
                    received,
                    start_byte
                )
            }
            Error::IdMismatch { packet, message } => {
                write!(f, "packet id is {}, message id is {}", packet, message)
            }
            Error::InvalidFormat(ref s) => write!(f, "packet is an invalid format: {}", s),
            Error::MessageComplete |
            Error::MissingTotalBytes |
            Error::NonExtendedContinuationPacket |
            Error::NonzeroStartByte => write!(f, "{}", self.description()),
            Error::ParseInt(ref err) => err.fmt(f),
            Error::UnsupportedPacketType(ref s) => write!(f, "unsupported packet type: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SELF_TIMED: &'static str = "0ATHB03313";
    const SELF_TIMED_EXTENDED_0: &'static str = include_str!("../../data/170801_000055.txt");
    const SELF_TIMED_EXTENDED_1: &'static str = include_str!("../../data/170801_000155.txt");
    const FORCED_TRANSMISSION: &'static str = include_str!("../../data/160719_193136.txt");

    #[test]
    fn message_add_self_timed() {
        let mut message = Message::new();
        message = message.add(SELF_TIMED).unwrap();
        assert!(message.is_complete());
        assert_eq!("ATHB03313", String::from(message));
    }

    #[test]
    fn message_add_self_timed_extended() {
        let mut message = Message::new();
        message = message.add(SELF_TIMED_EXTENDED_0).unwrap();
        assert!(!message.is_complete());
        message = message.add(SELF_TIMED_EXTENDED_1).unwrap();
        assert!(message.is_complete());
        assert_eq!(354, String::from(message.clone()).len());
        assert!(message.add(SELF_TIMED_EXTENDED_1).is_err());
    }

    #[test]
    fn forced_transmission() {
        match FORCED_TRANSMISSION.parse::<Packet>().unwrap() {
            Packet::ForcedTransmission(msg) => assert_eq!("test", msg),
            _ => panic!("Forced transmission was not recognized as such"),
        }
    }
}
