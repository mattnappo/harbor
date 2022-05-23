use crate::peer::{Key, PeerId, PeerStore};
use crate::{Error, NetworkError};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::net::{Ipv4Addr, TcpStream};

pub type NetworkResult<T> = Result<T, NetworkError>;

macro_rules! write_and_map {
    ($c:expr, $t:expr) => {
        $c.write($t).map_err(|e| NetworkError::Fail(e.to_string()))
    };
}

/// Possible peer request types
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// Respond with Pong!
    Ping,

    /// Responds with this peer's PeerId
    PeerId,

    /// Responds with this peer's list of stored files
    List,

    /// Responds with this peer's complete table of peers
    PeerStore,

    /// Asks this peer to add the given identity (id) to its table of peers
    Join { id: PeerId, ip: Ipv4Addr, port: u16 },

    /// Responds to the peer as to whether this peer has any recursive
    /// record of the given key in the given tts
    QueryKey { key: Key, tts: u16 },

    /// Notifies the peer that holding_id has a record of the given key
    RespondKey { holding_id: PeerId, key: Key },

    /// Request for this peer to send its copy the given key's value
    GetKey(Key),

    /// Sync this peer's peerstore with another peer's peerstore in the given tts
    SyncPeers { tts: u16 },

    /// Remove the given peer from this peer's table of peers
    Leave(PeerId),

    /// Respond with a message
    Msg(String),

    /// Respond with success
    Ok,

    /// Respond with an error
    Err(NetworkError),
}

/// A general protocol for this framework
pub trait Protocol {
    fn handle_ping(conn: &mut TcpStream, req: &Request) -> NetworkResult<usize>;
    fn handle_peer_id(conn: &mut TcpStream, req: &Request) -> NetworkResult<()>;
    fn handle_list(conn: &mut TcpStream, req: &Request) -> NetworkResult<()>;
    fn handle_peer_store(
        conn: &mut TcpStream,
        req: &Request,
        ps: &PeerStore,
    ) -> NetworkResult<usize>;
    fn handle_join(conn: &mut TcpStream, req: &Request) -> NetworkResult<()>;
    /* ... */
    fn handle_leave(conn: &mut TcpStream, req: &Request) -> NetworkResult<()>;
}

pub struct HarborProtocol;

impl Protocol for HarborProtocol {
    /// Handle an incoming Request::Ping
    fn handle_ping(conn: &mut TcpStream, req: &Request) -> NetworkResult<usize> {
        println!("writing pong");
        write_and_map!(conn, "Pong!".as_bytes())
    }
    fn handle_peer_id(conn: &mut TcpStream, req: &Request) -> NetworkResult<()> {
        Ok(())
    }
    fn handle_list(conn: &mut TcpStream, req: &Request) -> NetworkResult<()> {
        Ok(())
    }
    fn handle_peer_store(
        conn: &mut TcpStream,
        req: &Request,
        ps: &PeerStore,
    ) -> NetworkResult<usize> {
        println!("writing peer store");
        let ser = &bincode::serialize(ps)?[..];
        write_and_map!(conn, ser)
    }
    fn handle_join(conn: &mut TcpStream, req: &Request) -> NetworkResult<()> {
        Ok(())
    }
    /* ... */
    fn handle_leave(conn: &mut TcpStream, req: &Request) -> NetworkResult<()> {
        Ok(())
    }
}
