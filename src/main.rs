use harbor::peer;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut peer = peer::Peer::new(true, 3300).unwrap();
    peer.bootstrap()?;
    println!("new bootstrapped peer: {:?}", peer);
    peer.start().unwrap();

    Ok(())
}
