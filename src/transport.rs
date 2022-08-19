use crate::{
    peer::{Peer, PeerId},
    protocol::{NetworkResult, Request, Response},
    NetworkError,
};
use log::info;
use std::{
    io::prelude::*,
    net::{Shutdown, TcpStream},
    thread, time,
};

/// Send requests to a peer, and send responses back
pub trait Transport {
    fn send_request(to_peer: &PeerId, req: Request) -> NetworkResult<TcpStream>;
    fn send_response(conn: &mut TcpStream, res: Response) -> NetworkResult<usize>;
}

impl Transport for Peer {
    /// Send a request to a peer. The input PeerId `to_peer` should always
    /// be from the output of the routing function.
    fn send_request(to_peer: &PeerId, req: Request) -> NetworkResult<TcpStream> {
        // Dial the peer
        let mut conn = TcpStream::connect(to_peer.as_socket())?;
        info!("dialed peer {:?}", to_peer);

        // Assume, for now, that req is of type Request::Ping (why did i write this)
        let ser = &bincode::serialize(&req)?[..];

        conn.write(ser)?;
        info!("wrote request {req:?} to {to_peer:?}");
        Ok(conn)
    }

    /// Send a response to a request to the given TcpStream
    fn send_response(conn: &mut TcpStream, res: Response) -> NetworkResult<usize> {
        let ser = &bincode::serialize(&res)?[..];
        let status = conn.write(ser)?;
        info!("wrote response {res:?} to {conn:?}");
        Ok(status)
    }
}
