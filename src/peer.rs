use crate::protocol::*;
use crate::*;
use crate::{Error, NetworkError, MAX_PEERS};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net;
use std::net::Ipv4Addr;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// A key for a file
#[derive(Serialize, Deserialize, Debug)]
pub struct Key(String);

/// A String of the form ip:port
#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct PeerId(String);

impl std::cmp::PartialEq for PeerId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for PeerId {}

// TODO: Add a nonce field to this so that it differs
impl PeerId {
    pub fn from(ip: net::Ipv4Addr, port: u16) -> Self {
        Self(format!("{}:{}", ip, port))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub type PeerStore = HashMap<PeerId, (Ipv4Addr, u16)>;

/// A peer on the network
#[derive(Debug)]
pub struct Peer {
    port: u16,
    max_peers: u8,
    ip: Ipv4Addr,
    pub_ip: Option<Ipv4Addr>,
    local: bool,

    /// A map from PeerId to (ip, port) pairs
    peers: Arc<Mutex<PeerStore>>,
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
            peers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Add a peer to this peer's list of known peers
    pub fn add_peer(&mut self, new_peer: PeerId, ip: Ipv4Addr, port: u16) {
        let peers = self.peers.clone();
        let mut peers = peers.lock().unwrap();
        peers.insert(new_peer, (ip, port));
    }

    /// Start listening on this node
    pub fn start(&self) -> Result<(), Error> {
        let socket = TcpListener::bind(self.peer_id().to_string())?;

        for stream in socket.incoming() {
            self.handle_conn(stream?)?;
        }

        Ok(())
    }

    /// Read from the bootstrap file and add the bootstrap hosts to the PeerStore
    pub fn bootstrap(&mut self) -> Result<i32, Box<dyn std::error::Error>> {
        let mut c = 0i32; // Number of bootstrapped peers

        // Read each line from the bootstrap file
        if let Ok(lines) = util::read_lines(crate::BOOTSTRAP) {
            for line in lines {
                // For each host
                if let Ok(host) = line {
                    // Parse the ip and port and construct a PeerId
                    let data: Vec<String> =
                        host.split(":").map(|s| s.to_string()).collect();
                    if data.len() != 2 {
                        continue;
                    }
                    let ip = data[0].parse::<Ipv4Addr>()?;
                    let port: u16 = data[1].parse()?;

                    // Add the host
                    self.add_peer(PeerId::from(ip, port), ip, port);
                    c += 1;
                }
            }
        }
        Ok(c)
    }

    /// Handle an incoming connection
    fn handle_conn(&self, mut conn: TcpStream) -> Result<(), Error> {
        println!("handling new conn {:?}", conn);

        let peers = self.peers.clone();
        thread::spawn(move || -> Result<(), Error> {
            let mut buf = Vec::new();
            conn.read_to_end(&mut buf)?;

            let peers = peers.lock().unwrap();
            let request = bincode::deserialize::<Request>(&buf[..])?;
            match &request {
                Request::Ping => HarborProtocol::handle_ping(&mut conn, &request),
                Request::PeerStore => {
                    HarborProtocol::handle_peer_store(&mut conn, &request, &peers)
                }
                _ => todo!(),
            };
            Ok(())
        });

        Ok(())
    }

    /// Attempt to find a route to the given PeerId
    // TODO: Eventually make this recursive with a supplied depth??
    fn router(&self, peer: PeerId) -> Option<(PeerId, Ipv4Addr, u16)> {
        if let Some(ip_port) = self.peers.clone().lock().unwrap().get(&peer) {
            return Some((peer, ip_port.0, ip_port.1));
        }
        None
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
        let mut peer = Peer::new(true, 3300).unwrap();
        peer.bootstrap().unwrap();
        println!("peer: {:#?}", peer);
    }
}
