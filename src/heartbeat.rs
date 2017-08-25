/// A heartbeat from the ATLAS system.
///
/// These heartbeats are transmitted via Iridium SBD. Because of the SBD message length
/// restriction, heartbeats may come in one or more messages, and might have to be pieced together.
/// There are multiple version of the heartbeat messages, since Pete changes the format each time
/// he visits ATLAS.
#[derive(Clone, Copy, Debug)]
pub struct Heartbeat;
