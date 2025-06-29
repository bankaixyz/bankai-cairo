use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_rpc_types::Header;

use crate::{
    error::Error,
    fetch::{api::ApiClient, execution::ExecutionFetcher},
    proof_output::RecursiveEpochOutput,
    stone::verify::verify_proof,
};

pub struct VerifiedHeader(pub Header);

impl VerifiedHeader {
    pub async fn new(
        block_number: u64,
        api_client: &ApiClient,
        rpc_url: &str,
    ) -> Result<Self, Error> {
        let proof = api_client
            .fetch_proof_by_execution_height(block_number)
            .await?;

        let proof_output_felts = verify_proof(&proof)?;

        let epoch_output = RecursiveEpochOutput::from_proof_output(proof_output_felts)?;

        assert_eq!(
            block_number, epoch_output.execution_header_height,
            "Block number mismatch: expected {}, got {}",
            block_number, epoch_output.execution_header_height
        );

        let header_reader = ExecutionFetcher::new(rpc_url.to_string());
        let header = header_reader.fetch_header(block_number).await?;

        assert_eq!(
            header.hash_slow(),
            epoch_output.execution_header_root,
            "Header hash mismatch"
        );

        println!("RPC Header hash match!");

        Ok(Self(header))
    }

    pub fn parent_hash(&self) -> B256 {
        self.0.parent_hash
    }

    pub fn ommers_hash(&self) -> B256 {
        self.0.ommers_hash
    }

    pub fn beneficiary(&self) -> Address {
        self.0.beneficiary
    }

    pub fn state_root(&self) -> B256 {
        self.0.state_root
    }

    pub fn transactions_root(&self) -> B256 {
        self.0.transactions_root
    }

    pub fn receipts_root(&self) -> B256 {
        self.0.receipts_root
    }

    pub fn logs_bloom(&self) -> &[u8] {
        self.0.logs_bloom.as_slice()
    }

    pub fn difficulty(&self) -> U256 {
        self.0.difficulty
    }

    pub fn number(&self) -> u64 {
        self.0.number
    }

    pub fn gas_limit(&self) -> u64 {
        self.0.gas_limit
    }

    pub fn gas_used(&self) -> u64 {
        self.0.gas_used
    }

    pub fn timestamp(&self) -> u64 {
        self.0.timestamp
    }

    pub fn extra_data(&self) -> &Bytes {
        &self.0.extra_data
    }

    pub fn mix_hash(&self) -> B256 {
        self.0.mix_hash
    }

    pub fn nonce(&self) -> u64 {
        u64::from_be_bytes(self.0.nonce.0)
    }

    pub fn base_fee_per_gas(&self) -> Option<u64> {
        self.0.base_fee_per_gas
    }

    pub fn withdrawals_root(&self) -> Option<B256> {
        self.0.withdrawals_root
    }

    pub fn blob_gas_used(&self) -> Option<u64> {
        self.0.blob_gas_used
    }

    pub fn excess_blob_gas(&self) -> Option<u64> {
        self.0.excess_blob_gas
    }

    pub fn parent_beacon_block_root(&self) -> Option<B256> {
        self.0.parent_beacon_block_root
    }
}
