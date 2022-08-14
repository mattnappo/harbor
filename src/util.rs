use crate::Error;
use local_ip_address::local_ip;
use sha2::{Digest, Sha256, Sha512};
use std::{
    fs::File,
    io::{self, BufRead},
    net::{IpAddr, Ipv4Addr},
    path::Path,
};

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Hash a vector of bytes to a hex string using sha256
pub fn hash_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(&result)
}

/// Get this system's local ip address
pub fn get_local_ip() -> Result<Ipv4Addr, Error> {
    if let Ok(ip) = local_ip() {
        return match ip {
            IpAddr::V4(v4) => Ok(v4),
            IpAddr::V6(v6) => Err(Error::Ipv6Disabled(v6)),
        };
    }
    Err(Error::NoIp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let my_data = b"hello world";
        let hash = hash_sha256(my_data);
        println!("hash of {my_data:?}: {hash}");
    }
}
