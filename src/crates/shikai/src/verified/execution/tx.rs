// Now txs....

use alloy_primitives::{Address, Bytes, SignatureError, TxKind, B256, U256};
use alloy_rlp::Encodable;
use alloy_rpc_types::AccessList;
use alloy_trie::{proof::verify_proof, Nibbles};
use eth_trie_proofs::tx::ConsensusTx;

use crate::{
    error::Error,
    fetch::{api::ApiClient, execution::ExecutionFetcher},
    verified::execution::VerifiedHeader,
};

pub struct VerifiedTransaction(pub ConsensusTx);

impl VerifiedTransaction {
    pub async fn new(tx_hash: B256, api_client: &ApiClient, rpc_url: &str) -> Result<Self, Error> {
        let execution_fetcher = ExecutionFetcher::new(rpc_url.to_string());
        let block_number = execution_fetcher.fetch_tx_block_number(tx_hash).await?;
        let header = VerifiedHeader::new(block_number, api_client, rpc_url).await?;

        let tx_proof = execution_fetcher.fetch_tx_proof(tx_hash).await?;
        let tx_index = tx_proof.tx_index;

        let mut rlp_tx_index = Vec::new();
        tx_index.encode(&mut rlp_tx_index);
        let key = Nibbles::unpack(&rlp_tx_index);

        let proof_bytes: Vec<Bytes> = tx_proof
            .proof
            .iter()
            .map(|p| Bytes::from(p.clone()))
            .collect();

        verify_proof(
            header.transactions_root(),
            key,
            Some(tx_proof.encoded_tx.clone()),
            proof_bytes.iter(),
        )?;

        let tx = ConsensusTx::rlp_decode(tx_proof.encoded_tx.as_slice())?;

        Ok(Self(tx))
    }

    pub fn nonce(&self) -> u64 {
        self.0.nonce()
    }

    pub fn gas_limit(&self) -> u64 {
        self.0.gas_limit()
    }

    pub fn gas_price(&self) -> Option<u128> {
        self.0.gas_price()
    }

    pub fn to(&self) -> TxKind {
        self.0.to()
    }

    pub fn value(&self) -> U256 {
        self.0.value()
    }

    pub fn input(&self) -> Bytes {
        self.0.input().to_vec().into()
    }

    pub fn v(&self) -> u64 {
        self.0.v()
    }

    pub fn r(&self) -> U256 {
        self.0.r()
    }

    pub fn s(&self) -> U256 {
        self.0.s()
    }

    pub fn sender(&self) -> Result<Address, SignatureError> {
        self.0.sender()
    }

    pub fn chain_id(&self) -> Option<u64> {
        self.0.chain_id()
    }

    pub fn access_list(&self) -> Option<AccessList> {
        self.0.access_list()
    }

    pub fn max_fee_per_gas(&self) -> Option<u128> {
        self.0.max_fee_per_gas()
    }

    pub fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.0.max_priority_fee_per_gas()
    }

    pub fn blob_versioned_hashes(&self) -> Option<Vec<B256>> {
        self.0.blob_versioned_hashes()
    }

    pub fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.0.max_fee_per_blob_gas()
    }
}
