// verified beacon header here

use alloy_primitives::B256;
use tree_hash::TreeHash;

use crate::{
    error::Error,
    fetch::{
        api::ApiClient,
        beacon::{BeaconFetcher, BeaconHeader},
    },
    proof_output::RecursiveEpochOutput,
    stone::verify::verify_proof,
};

pub struct VerifiedBeaconHeader(pub BeaconHeader);

impl VerifiedBeaconHeader {
    pub async fn new(
        block_number: u64,
        api_client: &ApiClient,
        beacon_rpc_url: &str,
    ) -> Result<Self, Error> {
        let proof = api_client
            .fetch_proof_by_beacon_height(block_number)
            .await?;

        let proof_output_felts = verify_proof(&proof)?;

        let epoch_output = RecursiveEpochOutput::from_proof_output(proof_output_felts)?;

        assert_eq!(
            block_number, epoch_output.beacon_height,
            "Block number mismatch: expected {}, got {}",
            block_number, epoch_output.beacon_height
        );

        let beacon_fetcher = BeaconFetcher::new(beacon_rpc_url.to_string());
        let header = beacon_fetcher
            .fetch_header(epoch_output.beacon_height)
            .await?;

        assert_eq!(
            B256::from(header.tree_hash_root()),
            epoch_output.beacon_header_root,
            "Beacon header root mismatch"
        );

        println!("Beacon Header root match!");

        Ok(Self(header))
    }

    pub fn slot(&self) -> u64 {
        self.0.slot
    }

    pub fn proposer_index(&self) -> u64 {
        self.0.proposer_index
    }

    pub fn parent_root(&self) -> B256 {
        self.0.parent_root
    }

    pub fn state_root(&self) -> B256 {
        self.0.state_root
    }

    pub fn body_root(&self) -> B256 {
        self.0.body_root
    }
}
