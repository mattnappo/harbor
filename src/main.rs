use harbor::peer;

fn main() {
    let peer = peer::Peer::new(true, 3300).unwrap();
    peer.start().unwrap();
}
