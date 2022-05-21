use crate::peer::{Key, PeerId};
use crate::NetworkError;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// Possible peer request types
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// Respond with Pong!
    Ping,

    /// Responds with this peer's PeerId
    PeerId,

    /// Responds with this peer's list of stored files
    List,

    /// Asks this peer to add the given identity (id) to its table of peers
    Join { id: PeerId, ip: Ipv4Addr, port: u16 },

    /// Responds to the peer as to whether this peer has any recursive
    /// record of the given key in the given tts
    QueryKey { key: Key, tts: u16 },

    /// Notifies the peer that holding_id has a record of the given key
    RespondKey { holding_id: PeerId, key: Key },

    /// Request for this peer to send its copy the given key's value
    GetKey(Key),

    /// Remove the given peer from this peer's table of peers
    Leave(PeerId),

    /// Respond with a message
    Msg(String),

    /// Respond with success
    Ok,

    /// Respond with an error
    Err(NetworkError),
}

/* DO THIS LATER
/// A general protocol for this framework
pub trait Protocol {
    pub fn handle_ping(TcpStream);
}

struct HarborProtocol;

impl Protocol for HarborProtocol {}
*/
