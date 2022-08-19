use crate::{
    protocol::Protocol,
    protocol::*,
    transport::Transport,
    util, {Error, NetworkError, MAX_PEERS},
};
use chrono;
use derivative::Derivative;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt,
    io::prelude::*,
    net::{IpAddr, Ipv4Addr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

/// A key for a file
#[derive(Serialize, Deserialize, Debug)]
pub struct Key(String);

/// A unique identifier for peers on the network based on libp2p's
/// multiaddr
#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct PeerId {
    id: String,
    ip: Ipv4Addr,
    port: u16,
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl std::cmp::PartialEq for PeerId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::PartialEq for PeerStoreEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PeerId {}
impl Eq for PeerStoreEntry {}

impl PeerId {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        // TODO: once encryption is added, the hash will be of peer's pubkey
        let data = format!("{ip}:{port}");
        let hash = util::hash_sha256(data.as_bytes());
        Self {
            id: format!("/peer/{hash}/{ip}/{port}"),
            ip,
            port,
        }
    }

    pub fn from(ip: Ipv4Addr, port: u16) -> Self {
        PeerId::new(ip, port)
    }

    pub fn to_string(&self) -> String {
        self.id.clone()
    }

    /// Return this PeerId in the format ip:port
    pub fn as_socket(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    /// Return this PeerId's ip
    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    /// Return this PeerId's port
    pub fn port(&self) -> u16 {
        self.port
    }
}

/// An entry in a PeerStore
#[derive(Derivative, Debug, Serialize, Deserialize, Clone)]
#[derivative(Hash)]
pub struct PeerStoreEntry {
    #[derivative(Hash = "ignore")]
    last_seen: Option<chrono::NaiveDateTime>,
    id: PeerId,
}

impl PeerStoreEntry {
    pub fn new(id: PeerId) -> Self {
        Self {
            last_seen: None,
            id,
        }
    }
}

pub type PeerStore = HashSet<PeerStoreEntry>;

/// A peer on the network. This represents the peer running on this machine
#[derive(Debug)]
pub struct Peer {
    pub(crate) id: PeerId,
    max_peers: u8,
    pub_ip: Option<Ipv4Addr>, // Deprecated
    local: bool,

    /// A map from PeerId to (ip, port) pairs
    pub(crate) peers: Arc<Mutex<PeerStore>>,
}

impl Peer {
    /// Construct a new peer
    pub fn new(local: bool, port: u16) -> Result<Self, Error> {
        Ok(Self {
            id: PeerId::from(util::get_local_ip()?, port),
            max_peers: MAX_PEERS,
            pub_ip: None,
            local,
            peers: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    /// Add a peer to this peer's list of known peers
    pub fn add_peer(&mut self, new_peer: PeerId) -> bool {
        let peers = self.peers.clone();
        let mut peers = peers.lock().unwrap();

        // Cannot store ourself in the PeerStore
        if new_peer == self.id {
            return false;
        }
        peers.insert(PeerStoreEntry::new(new_peer))
    }

    /// Start listening on this peer
    /// TODO: Run a grpc server (async?) to run local client API to
    /// interface with the node
    pub fn start(mut self, send_pings: bool) -> Result<(), Error> {
        self.bootstrap()?; // Bootstrap this peer

        // This loop will run forever
        // TODO: Handle incoming connections in a separate thread
        let socket = TcpListener::bind(&self.id.as_socket())?;
        info!("starting peer {:#?}", self);
        info!("bound peer on socket {:?}", self.id.as_socket());

        // TODO: Delete, replace with RPC
        if send_pings {
            self.send_pings()?;
        }

        loop {
            info!("listening for incoming connections");
            // Listen for new incoming connections (requests)
            for stream in socket.incoming() {
                self = self.handle_conn(stream?)?;
            }
        }
    }

    /// Read from the bootstrap file and add the bootstrap hosts to the PeerStore
    fn bootstrap(&mut self) -> Result<i32, Error> {
        let mut count = 0i32; // Number of bootstrapped peers

        // Read each line from the bootstrap file
        if let Ok(lines) = util::read_lines(crate::BOOTSTRAP_FILE) {
            for line in lines {
                // For each host
                if let Ok(host) = line {
                    // Parse the ip and port and construct a PeerId
                    let data: Vec<String> =
                        host.split(":").map(|s| s.to_string()).collect();
                    if data.len() != 2 {
                        continue;
                    }
                    let ip = data[0].parse::<Ipv4Addr>().unwrap();
                    let port: u16 = data[1].parse().unwrap();
                    let id = PeerId::from(ip, port);

                    count += self.add_peer(id) as i32;
                }
            }
        }

        Ok(count)
    }

    /// Handle a new incoming connection (a request)
    /// TOOD: convert this function into async
    fn handle_conn(mut self, mut conn: TcpStream) -> Result<Self, Error> {
        let peers = self.peers.clone();
        thread::spawn(move || -> Result<Self, Error> {
            let mut buf = vec![0u8; MAX_TRANSFER_SIZE];
            let len = conn.read(&mut buf)?;
            let request = bincode::deserialize::<Request>(&buf[0..len]).unwrap();

            info!("handling request {request:?} from {conn:?}");

            // Call the handlers defined in Protocol impl
            match request {
                Request::Ping => {
                    self.handle_ping(&mut conn)?;
                }
                Request::Identity => {
                    self.handle_identity(&mut conn)?;
                }
                Request::Join(id) => {
                    self.handle_join(&mut conn, id)?;
                }
                Request::PeerStore => {
                    self.handle_peerstore(&mut conn)?;
                }
                _ => todo!(),
            }

            Ok(self)
        })
        .join()
        .unwrap()
    }

    /// Handle a response
    fn handle_response(&self, mut conn: TcpStream) -> Result<(), Error> {
        info!("handling response from conn {conn:?}");

        thread::spawn(move || -> Result<(), Error> {
            let mut buf = Vec::new();
            conn.read_to_end(&mut buf)?; // Can read to end because socket closes

            let response = bincode::deserialize::<Response>(&buf[..]).unwrap();
            info!("handling response {response:?} from conn {conn:?}");

            // Call the handlers defined in Protocol impl
            match &response {
                Response::Pong => info!("got a pong from {conn:?}!"),
                _ => todo!(),
            };
            Ok(())
        })
        .join()
        .unwrap()
    }

    /// Attempt to find a route to the given PeerId
    // TODO: Eventually make this recursive with a supplied depth??
    fn router(&self, peer: PeerId) -> Option<PeerId> {
        // If the desired peer is us, return ourself
        if peer == self.id {
            return Some(self.id.clone());
        }

        // If not, check if the desired peer is in our PeerStore, and
        // return it
        self.peers
            .clone()
            .lock()
            .unwrap()
            .get(&PeerStoreEntry::new(peer))
            .cloned()
            .map(|p| p.id)
    }

    /* Public functions define interface to Peer */

    /// Send a ping to all nodes in the peerstore
    pub fn send_pings(&self) -> Result<(), Error> {
        let inner_peers = self.peers.clone();
        let peers = inner_peers.lock().unwrap();
        for id in peers.iter().map(|peer| peer.id.clone()) {
            self.send_ping(&id)?;
        }
        Ok(())
    }

    /// Send a ping request to a peer
    pub fn send_ping(&self, to: &PeerId) -> Result<(), Error> {
        let conn = Peer::send_request(&to, Request::Ping)?;
        self.handle_response(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id() {
        let id1 = PeerId::from("127.0.0.1".parse().unwrap(), 3300);
        println!("{id1:?}");
        assert!(id1.ip() == id1.ip);
        assert!(id1.port() == id1.port);
    }

    #[test]
    fn test_bootstrap() {
        let mut peer = Peer::new(true, 3300).unwrap();
        peer.bootstrap().unwrap();
        println!("peer: {:#?}", peer);
    }

    #[test]
    fn add_peer() {
        let mut peer = Peer::new(true, 9900).unwrap();

        peer.add_peer(PeerId::from("127.0.0.1".parse().unwrap(), 3300));
        peer.add_peer(PeerId::from("127.0.0.1".parse().unwrap(), 3300));
        peer.add_peer(PeerId::from("192.168.1.12".parse().unwrap(), 9954));
        peer.add_peer(PeerId::from("192.168.1.82".parse().unwrap(), 9900));

        println!("{peer:#?}");
    }
}
