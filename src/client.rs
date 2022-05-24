use crate::peer::{Peer, PeerId};
use crate::protocol::write_and_map;
use crate::protocol::{NetworkResult, Request, Response};
use crate::*;
use std::io::prelude::*;
use std::net::TcpStream;

pub trait Client {
    fn send_request(to_peer: PeerId, req: Request) -> NetworkResult<usize>;
    fn send_response(to_peer: PeerId, res: Response) -> NetworkResult<usize>;
}

impl Client for Peer {
    /// Send a request to a peer. The input PeerId `to_peer` should always
    /// be from the output of the routing function.
    fn send_request(to_peer: PeerId, req: Request) -> NetworkResult<usize> {
        // Dial the peer
        let mut stream = TcpStream::connect(to_peer.to_string())?;

        println!("writing req {:#?} to {:?}", req, to_peer);

        // Assume, for now, that req is of type Request::Ping
        let ser = &bincode::serialize(&req)?[..];

        write_and_map!(stream, ser)
    }

    fn send_response(to_peer: PeerId, res: Response) -> NetworkResult<usize> {
        todo!();
    }
}
