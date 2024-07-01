use url::Url;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use crate::domain::entities::info::Info;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent
{
    #[serde_as(as = "DisplayFromStr")]
    pub announce: Url,
    pub info: Info,
}