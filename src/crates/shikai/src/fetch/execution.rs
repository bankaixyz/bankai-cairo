use alloy_primitives::{Address, FixedBytes};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types::{EIP1186AccountProofResponse, Header as ExecutionHeader};
use eth_trie_proofs::tx_trie::TxsMptHandler;
use url::Url;

use crate::error::Error;

pub struct ExecutionFetcher {
    pub base_url: String,
}

pub struct TxProof {
    pub tx_index: u64,
    pub proof: Vec<Vec<u8>>,
    pub encoded_tx: Vec<u8>,
}

impl ExecutionFetcher {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn fetch_header(&self, block_number: u64) -> Result<ExecutionHeader, Error> {
        let rpc_url: Url = self.base_url.parse()?;
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let block = provider
            .get_block_by_number(block_number.into())
            .await
            .map_err(|_| Error::BlockNotFound)?
            .unwrap();

        Ok(block.header)
    }

    pub async fn fetch_account_proof(
        &self,
        address: Address,
        block_number: u64,
    ) -> Result<EIP1186AccountProofResponse, Error> {
        let rpc_url: Url = self.base_url.parse()?;
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let proof = provider
            .get_proof(address, vec![])
            .block_id(block_number.into())
            .await
            .map_err(|_| Error::BlockNotFound)?;

        Ok(proof)
    }

    pub async fn fetch_tx_block_number(&self, tx_hash: FixedBytes<32>) -> Result<u64, Error> {
        let rpc_url: Url = self.base_url.parse()?;
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let receipt = provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|_| Error::BlockNotFound)?
            .ok_or(Error::BlockNotFound)?;

        Ok(receipt.block_number.unwrap())
    }

    pub async fn fetch_tx_proof(&self, tx_hash: FixedBytes<32>) -> Result<TxProof, Error> {
        let rpc_url: Url = self.base_url.parse()?;

        let mut txs_mpt_handler = TxsMptHandler::new(rpc_url).unwrap();
        txs_mpt_handler
            .build_tx_tree_from_tx_hash(tx_hash)
            .await
            .unwrap();

        let tx_index = txs_mpt_handler.tx_hash_to_tx_index(tx_hash).unwrap();
        let proof = txs_mpt_handler.get_proof(tx_index).unwrap();
        let encoded_tx = txs_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .unwrap();

        println!("Tx proof verified successfully!");

        Ok(TxProof {
            tx_index,
            proof,
            encoded_tx,
        })
    }
}
