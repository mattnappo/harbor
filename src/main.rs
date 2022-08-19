use harbor::peer;
use std::{env, error::Error};

fn peer(port: u16) -> Result<(), Box<dyn Error>> {
    let peer = peer::Peer::new(true, port)?;

    // If bootstrap peer, don't send pings
    if port == 3300 {
        peer.start(false)?;
    } else {
        peer.start(true)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let port = args[1].parse::<u16>()?;
        return peer(port);
    } else {
        panic!("provide a port");
    }
}
