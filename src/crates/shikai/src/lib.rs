use alloy_primitives::{Address, FixedBytes};

use crate::{
    error::Error,
    fetch::api::ApiClient,
    verified::{
        beacon::VerifiedBeaconHeader,
        execution::{account::VerifiedAccount, tx::VerifiedTransaction, VerifiedHeader},
    },
};

pub mod error;
pub mod fetch;
pub mod proof_output;
pub mod stone;
pub mod verified;

pub struct Shikai {
    pub api_client: ApiClient,
    pub execution_rpc: String,
    pub beacon_rpc: String,
}

pub struct Execution<'a> {
    api_client: &'a ApiClient,
    rpc_url: &'a str,
}

impl Execution<'_> {
    pub async fn header(&self, block_number: u64) -> Result<VerifiedHeader, Error> {
        VerifiedHeader::new(block_number, self.api_client, self.rpc_url).await
    }

    pub async fn account(
        &self,
        address: Address,
        block_number: u64,
    ) -> Result<VerifiedAccount, Error> {
        VerifiedAccount::new(address, block_number, self.api_client, self.rpc_url).await
    }

    pub async fn tx(&self, tx_hash: FixedBytes<32>) -> Result<VerifiedTransaction, Error> {
        VerifiedTransaction::new(tx_hash, self.api_client, self.rpc_url).await
    }
}

pub struct Beacon<'a> {
    api_client: &'a ApiClient,
    rpc_url: &'a str,
}

impl Beacon<'_> {
    pub async fn header(&self, block_number: u64) -> Result<VerifiedBeaconHeader, Error> {
        VerifiedBeaconHeader::new(block_number, self.api_client, self.rpc_url).await
    }
}

impl Shikai {
    pub fn new(execution_rpc: String, beacon_rpc: String) -> Self {
        let api_client = ApiClient::new("http://127.0.0.1:3030".to_string());
        Self {
            api_client,
            execution_rpc,
            beacon_rpc,
        }
    }

    pub fn execution(&self) -> Execution<'_> {
        Execution {
            api_client: &self.api_client,
            rpc_url: &self.execution_rpc,
        }
    }

    pub fn beacon(&self) -> Beacon<'_> {
        Beacon {
            api_client: &self.api_client,
            rpc_url: &self.beacon_rpc,
        }
    }
}
