use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MpcNodeAddresses {
    pub node_nr: u32,
    pub address: String,
}
