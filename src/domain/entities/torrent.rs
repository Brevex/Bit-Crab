use crate::domain::entities::info::Info;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use url::Url;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    #[serde_as(as = "DisplayFromStr")]
    pub announce: Url,
    pub info: Info,
}
