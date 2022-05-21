#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod peer;
pub mod protocol;

const MAX_PEERS: u8 = 32;

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// Some general error that happened on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    /// An unknown network failure
    Fail,

    /// A network io error
    IoError,
}

/// The general crate error
#[derive(Debug)]
pub enum Error {
    NoIp,
    Ipv6Disabled(std::net::Ipv6Addr),
    IoError(std::io::Error),
    BinaryError(bincode::Error),
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
            Error::IoError(e) => write!(f, "{:?}", e),
            _ => write!(f, "other error"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::NoIp => None,
            Error::Ipv6Disabled(ip) => None,
            Error::IoError(ref e) => Some(e),
            Error::BinaryError(ref e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Error {
        Error::BinaryError(err)
    }
}
