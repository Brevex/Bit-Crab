use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_peer_id() -> [u8; 20] {
    let mut peer_id = [0u8; 20];
    let prefix = b"-CRAB01-";
    peer_id[..8].copy_from_slice(prefix);

    let mut rng = rand::thread_rng();
    for byte in &mut peer_id[8..] {
        *byte = rng.sample(Alphanumeric);
    }
    peer_id
}
