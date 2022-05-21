use crate::protocol::*;
use crate::{Error, NetworkError, MAX_PEERS};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net;
use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::thread;

/// A key for a file
#[derive(Serialize, Deserialize, Debug)]
pub struct Key(String);

/// A String of the form ip:port
#[derive(Serialize, Deserialize, Debug)]
pub struct PeerId(String);

// TODO: Add a nonce field to this so that it differs
impl PeerId {
    fn from(ip: net::Ipv4Addr, port: u16) -> Self {
        Self(format!("{}:{}", ip, port))
    }

    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// A peer on the netwo:qrk
#[derive(Debug)]
pub struct Peer {
    port: u16,
    max_peers: u8,
    ip: Ipv4Addr,
    pub_ip: Option<Ipv4Addr>,
    local: bool,

    /// A map from PeerId to (ip, port) pairs
    peers: HashMap<PeerId, (Ipv4Addr, u16)>,
}

impl Peer {
    /// Construct a new peer
    pub fn new(local: bool, port: u16) -> Result<Self, Error> {
        Ok(Self {
            port,
            max_peers: MAX_PEERS,
            ip: Self::get_local_ip()?,
            pub_ip: None,
            local,
            peers: HashMap::new(),
        })
    }

    /// Start listening on this node
    pub fn start(&self) -> Result<(), Error> {
        let socket = TcpListener::bind(self.peer_id().to_string())?;

        for stream in socket.incoming() {
            self.handle_conn(stream?)?;
        }

        Ok(())
    }

    /// Handle an incoming connection
    fn handle_conn(&self, mut conn: TcpStream) -> Result<(), Error> {
        println!("handling new conn {:?}", conn);

        let handle = thread::spawn(move || -> Result<(), Error> {
            let mut buf = Vec::new();
            conn.read_to_end(&mut buf)?;

            let request = bincode::deserialize::<Request>(&buf[..])?;
            match &request {
                Request::Ping => Self::handle_ping(&mut conn, &request),
                _ => todo!(),
            };
            Ok(())
        });

        match handle.join() {
            Ok(_) => return Ok(()),
            Err(e) => println!("{:?}", e),
        };

        Ok(())
    }

    fn handle_ping(conn: &mut TcpStream, request: &Request) -> Result<(), Error> {
        println!("writing pong");
        conn.write("Pong!".as_bytes()).unwrap();
        Ok(())
    }

    /// Get the `PeerId` for this peer
    fn peer_id(&self) -> PeerId {
        if self.local {
            return PeerId::from(self.ip, self.port);
        }

        match self.pub_ip {
            Some(ip) => PeerId::from(ip, self.port),
            None => PeerId::from(self.ip, self.port),
        }
    }

    /// Get this system's local ip address
    fn get_local_ip() -> Result<Ipv4Addr, Error> {
        if let Ok(ip) = local_ip() {
            return match ip {
                net::IpAddr::V4(v4) => Ok(v4),
                net::IpAddr::V6(v6) => Err(Error::Ipv6Disabled(v6)),
            };
        }
        Err(Error::NoIp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_peer() {
        let peer = Peer::new(true, 3300);
        println!("peer: {:#?}", peer);
    }
}
