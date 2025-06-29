use alloy_consensus::Account;
use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_rlp::Encodable;
use alloy_trie::{proof::verify_proof, Nibbles};

use crate::{error::Error, fetch::api::ApiClient};

#[derive(Debug)]
pub struct VerifiedAccount(pub Account);

impl VerifiedAccount {
    pub async fn new(
        address: Address,
        block_number: u64,
        api_client: &ApiClient,
        rpc_url: &str,
    ) -> Result<Self, Error> {
        // Get the header from trustless bankai proof
        let header =
            crate::verified::execution::VerifiedHeader::new(block_number, api_client, rpc_url)
                .await?;
        let mpt_proof = crate::fetch::execution::ExecutionFetcher::new(rpc_url.to_string())
            .fetch_account_proof(address, block_number)
            .await?;

        let key = Nibbles::unpack(keccak256(address));
        let account = Account {
            nonce: mpt_proof.nonce,
            balance: mpt_proof.balance,
            storage_root: mpt_proof.storage_hash,
            code_hash: mpt_proof.code_hash,
        };
        let mut expected_value = Vec::new();
        account.encode(&mut expected_value);

        verify_proof(
            header.state_root(),
            key,
            Some(expected_value),
            mpt_proof.account_proof.iter(),
        )?;

        Ok(VerifiedAccount(account))
    }

    pub fn nonce(&self) -> u64 {
        self.0.nonce
    }

    pub fn balance(&self) -> U256 {
        self.0.balance
    }

    pub fn storage_root(&self) -> B256 {
        self.0.storage_root
    }

    pub fn code_hash(&self) -> B256 {
        self.0.code_hash
    }
}
