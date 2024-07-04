use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub length: usize,
    pub path: Vec<String>,
}
