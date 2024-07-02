use rand::Rng;
use rand::distributions::Alphanumeric;

pub fn generate_peer_id() -> String
{
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect()
}
