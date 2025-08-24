use thiserror::Error;

pub mod atlantic;
pub mod beacon_chain;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Beacon chain error: {0}")]
    Beacon(#[from] beacon_chain::BeaconError),
    #[error("Atlantic error: {0}")]
    Atlantic(#[from] atlantic::AtlanticError),
}
