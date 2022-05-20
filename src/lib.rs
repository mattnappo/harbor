pub mod peer;

const MAX_PEERS: u8 = 32;

use std::fmt;

/// Some general error that happened on the network
#[derive(Debug, Clone)]
pub enum NetworkError {
    Fail,
}

/// The general crate error
#[derive(Debug, Clone)]
pub enum Error {
    NoIp,
    Ipv6Disabled(std::net::Ipv6Addr),
}

#[derive(Debug, Clone)]
struct NoIpError;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoIp => {
                write!(f, "public or private ip cannot be found for this peer")
            }
            Error::Ipv6Disabled(ip) => {
                write!(f, "ipv6 ip {} found, but ipv6 is disabled", ip)
            }
        }
    }
}
