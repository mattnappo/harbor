use crate::{peer::*, transport::Transport, Error, NetworkError};
use serde::{Deserialize, Serialize};
use std::{
    io::prelude::*,
    net::{Ipv4Addr, TcpStream},
};

pub type NetworkResult<T> = Result<T, NetworkError>;

/// The maximum size of data that can be in a request or response over
/// the network
pub const MAX_TRANSFER_SIZE: usize = 5096; // in bytes

/// Possible peer request types
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// Ping this peer
    /// Responds with Response::Pong
    Ping,

    /// Ask this peer for its PeerId
    /// Responds with Response::PeerId
    PeerId,

    /// Asks this peer for its list of stored files
    /// Responds with Response::List
    List,

    /// Ask for this peer's PeerStore
    /// Responds Response::PeerStore
    PeerStore,

    /// Asks this peer to add the given identity (id) to its table of peers
    /// Responds with Response::Msg or Response::Err
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Respond with success
    Ok,

    /// Respond to a `Request::Ping`
    Pong,

    /// Respond with an error
    Err(NetworkError),

    /// Respond with a string message
    Msg(String),

    /// A complete PeerStore response
    /// Is a result of Request::PeerStore
    PeerStore(PeerStore),
}

/// A general protocol for this framework
pub trait Protocol {
    fn handle_ping(conn: &mut TcpStream, req: &Request) -> NetworkResult<usize>;
    fn handle_peer_id(conn: &mut TcpStream, req: &Request) -> NetworkResult<()>;
    fn handle_list(conn: &mut TcpStream, req: &Request) -> NetworkResult<()>;
    fn handle_peer_store(
        conn: &mut TcpStream,
        req: &Request,
        ps: &PeerStore, // Band-aid solution
    ) -> NetworkResult<usize>;
    fn handle_join(
        conn: &mut TcpStream,
        req: &Request,
        id: PeerId,
        ip: Ipv4Addr,
        port: u16,
        ps: &mut PeerStore,
    ) -> NetworkResult<usize>;
    /* ... */
    fn handle_leave(conn: &mut TcpStream, req: &Request) -> NetworkResult<()>;
}

impl Protocol for Peer {
    /// Handle an incoming Request::Ping
    fn handle_ping(conn: &mut TcpStream, req: &Request) -> NetworkResult<usize> {
        println!("writing pong");
        let len = Peer::send_response(conn, Response::Pong)?;
        println!("finished writing pong");
        Ok(len)
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
        //let ser = &bincode::serialize(ps)?[..];

        //write_and_map!(conn, &Response::PeerStore(ps.to_owned()))
        Peer::send_response(conn, Response::PeerStore(ps.to_owned()))
    }

    fn handle_join(
        conn: &mut TcpStream,
        req: &Request,
        id: PeerId,
        ip: Ipv4Addr,
        port: u16,
        ps: &mut PeerStore,
    ) -> NetworkResult<usize> {
        if ps.insert(id) {
            return Peer::send_response(
                conn,
                Response::Err(NetworkError::Fail("peer already joined".to_string())),
            );
        }
        Peer::send_response(conn, Response::Msg("join success".to_string()))
    }

    /* ... */

    fn handle_leave(conn: &mut TcpStream, req: &Request) -> NetworkResult<()> {
        Ok(())
    }
}
