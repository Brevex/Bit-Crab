use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peer
{
    pub id: Uuid,
    pub ip: String,
    pub port: u16,
}
