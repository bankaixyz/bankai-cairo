use std::env;

use crate::{
    clients::{atlantic::AtlanticClient, beacon_chain::BeaconRpcClient},
    db::Database,
    utils::config::BankaiConfig,
};

pub mod clients;
pub mod conversion;
pub mod db;
pub mod fetcher;
pub mod utils;
use dotenv::from_filename;

#[derive(Debug)]
pub struct BankaiClient {
    pub client: BeaconRpcClient,
    // pub config: BankaiConfig,
    pub db: Database,
    pub atlantic_client: AtlanticClient,
}

impl BankaiClient {
    pub async fn new(is_docker: bool) -> Self {
        let config = if is_docker {
            BankaiConfig::docker_config()
        } else {
            from_filename(".env.sepolia").ok();
            BankaiConfig::default()
        };

        let db = Database::new(&config.database_url)
            .await
            .expect("Failed to initialize database");

        Self {
            client: BeaconRpcClient::new(env::var("BEACON_RPC_URL").unwrap(), config.clone()),
            atlantic_client: AtlanticClient::new(
                config.atlantic_endpoint.clone(),
                env::var("ATLANTIC_API_KEY").unwrap(),
            ),
            db,
            // config,
        }
    }
}
