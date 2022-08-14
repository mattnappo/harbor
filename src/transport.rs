use crate::{
    peer::{Peer, PeerId},
    protocol::{NetworkResult, Request, Response},
    NetworkError,
};
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
        let mut conn = TcpStream::connect(to_peer.to_string())?;
        println!("dialed addr: {:?}", to_peer.to_string());
        println!("built connection {:?}", conn);

        // Assume, for now, that req is of type Request::Ping (why did i write this)
        let ser = &bincode::serialize(&req)?[..];

        println!("writing req {:#?} to {:?}\nraw: {:?}", req, to_peer, ser);

        conn.write(ser)?;
        Ok(conn)
    }

    /// Send a response to a request to the given TcpStream
    fn send_response(conn: &mut TcpStream, res: Response) -> NetworkResult<usize> {
        let ser = &bincode::serialize(&res)?[..];

        println!("writing res {:#?} to {:?}\nraw: {:?}", res, conn, ser);

        conn.write(ser) // SAME CONN
            .map_err(|e| NetworkError::Fail(e.to_string()))
    }
}
