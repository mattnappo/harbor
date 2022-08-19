use crate::{peer::*, transport::Transport, Error, NetworkError};
use log::warn;
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
    Identity,

    /// Asks this peer for its list of stored files
    /// Responds with Response::List
    List,

    /// Ask for this peer's PeerStore
    /// Responds Response::PeerStore
    PeerStore,

    /// Asks this peer to add the given identity (id) to its table of peers
    /// Responds with Response::Ok or Response::Err
    Join(PeerId),

    /// Responds to the peer as to whether this peer has any recursive
    /// record of the given key in the given tts
    QueryKey { key: Key, tts: u16 },

    /// Notifies the peer that holding_id has a record of the given key
    RespondKey { holding_id: PeerId, key: Key },

    /// Request for this peer to send its copy the given key's value
    Get(Key),

    /// Sync this peer's peerstore with another peer's peerstore in the given tts
    SyncPeers { tts: u16 },

    /// Remove the given peer from this peer's table of peers
    Leave(PeerId),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    /// Respond with success
    Ok,

    /// Respond with an error
    Err(NetworkError),

    /// Respond with a string message
    Msg(String),

    /* each variant corresponds to a request */
    /// Respond to a `Request::Ping`
    Pong,

    /// A response containing a single PeerId
    Identity(PeerId),

    /// Responds with a list of this peer's stored files
    List(Vec<Key>),

    /// Respond with this Peer's complete PeerStore
    /// Responds to Request::PeerStore
    PeerStore(PeerStore),
}

/// A general protocol for this framework
/*
    Ping
    Identity
    List
    PeerStore
    Join
    QueryKey
    RespondKey
    Get
    SyncPeers
    Leave
*/

pub trait Protocol {
    fn handle_ping(&self, conn: &mut TcpStream) -> NetworkResult<usize>;
    fn handle_identity(&self, conn: &mut TcpStream) -> NetworkResult<usize>;
    fn handle_list(&self, conn: &mut TcpStream) -> NetworkResult<usize>;
    fn handle_peerstore(&self, conn: &mut TcpStream) -> NetworkResult<usize>;
    fn handle_join(
        &mut self,
        conn: &mut TcpStream,
        new_peer: PeerId,
    ) -> NetworkResult<usize>;
    /* ... */
    fn handle_leave(&self, conn: &mut TcpStream) -> NetworkResult<usize>;
}

impl Protocol for Peer {
    /// Handle an incoming Request::Ping
    fn handle_ping(&self, conn: &mut TcpStream) -> NetworkResult<usize> {
        Peer::send_response(conn, Response::Pong)
    }

    /// Handle an incoming Request::Identity
    fn handle_identity(&self, conn: &mut TcpStream) -> NetworkResult<usize> {
        Peer::send_response(conn, Response::Identity(self.id.clone()))
    }

    /// Return a list of keys stored on this peer
    fn handle_list(&self, conn: &mut TcpStream) -> NetworkResult<usize> {
        Peer::send_response(
            conn,
            Response::Err(NetworkError::Fail("no keys stored yet".to_string())),
        )
    }

    /// Return this peer's entire PeerStore
    fn handle_peerstore(&self, conn: &mut TcpStream) -> NetworkResult<usize> {
        let peers = self.peers.clone();
        let peers = peers.lock().unwrap();
        Peer::send_response(conn, Response::PeerStore(peers.clone()))
    }

    /// Request to join this peer's PeerStore
    fn handle_join(
        &mut self,
        conn: &mut TcpStream,
        new_peer: PeerId,
    ) -> NetworkResult<usize> {
        if self.add_peer(self.id.clone()) {
            return Peer::send_response(
                conn,
                Response::Err(NetworkError::Fail("peer already joined".to_string())),
            );
        }
        Peer::send_response(conn, Response::Msg("join success".to_string()))
    }

    /* ... */

    fn handle_leave(&self, conn: &mut TcpStream) -> NetworkResult<usize> {
        Ok(0)
    }
}
