use alloy_primitives::FixedBytes;
use beacon_state_proof::state_proof_fetcher::{StateProofFetcher, SyncCommitteeProof, TreeHash};
use bls12_381::{G1Affine, G1Projective};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::utils::hashing::get_committee_hash;

/// Represents the public keys of sync committee validators and their aggregate
#[derive(Debug, Clone)]
pub struct SyncCommitteeValidatorPubs {
    /// Individual public keys of all validators in the committee
    pub validator_pubs: Vec<G1Affine>,
    /// Aggregated public key of all validators combined
    pub aggregate_pub: G1Affine,
}

/// Contains sync committee update data for epoch transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCommitteeData {
    /// Beacon chain slot number
    pub beacon_slot: u64,
    /// Merkle branch for next sync committee
    pub next_sync_committee_branch: Vec<FixedBytes<32>>,
    /// Aggregated public key of next sync committee
    pub next_aggregate_sync_committee: FixedBytes<48>,
    /// Root hash of committee keys
    pub committee_keys_root: FixedBytes<32>,
}

impl SyncCommitteeData {
    /// Creates a new sync committee update for a given slot
    pub async fn new(
        client: &crate::clients::beacon_chain::BeaconRpcClient,
        slot: u64,
    ) -> Result<SyncCommitteeData, SyncCommitteeError> {
        let state_proof_fetcher = StateProofFetcher::new(client.rpc_url.clone());
        let proof = state_proof_fetcher
            .fetch_next_sync_committee_proof(slot)
            .await?;

        Ok(SyncCommitteeData::from(proof))
    }

    /// Computes the state root by hashing the committee keys root and the aggregate pubkey
    pub fn compute_state_root(&self) -> FixedBytes<32> {
        let mut padded_aggregate = vec![0u8; 64];
        padded_aggregate[..48].copy_from_slice(&self.next_aggregate_sync_committee[..]);
        let aggregate_root: FixedBytes<32> =
            FixedBytes::from_slice(&Sha256::digest(&padded_aggregate));

        let mut leaf_data = [0u8; 64];
        leaf_data[0..32].copy_from_slice(self.committee_keys_root.as_slice());
        leaf_data[32..64].copy_from_slice(aggregate_root.as_slice());
        let leaf = FixedBytes::from_slice(&Sha256::digest(leaf_data));

        crate::utils::merkle::sha256::hash_path(self.next_sync_committee_branch.clone(), leaf, 55)
    }
}

impl From<SyncCommitteeProof> for SyncCommitteeData {
    fn from(committee_proof: SyncCommitteeProof) -> Self {
        let committee_keys_root = &committee_proof.next_sync_committee.pubkeys.tree_hash_root();

        Self {
            beacon_slot: committee_proof.slot,
            next_sync_committee_branch: committee_proof
                .proof
                .into_iter()
                .map(|bytes| FixedBytes::from_slice(bytes.as_slice()))
                .collect(),
            next_aggregate_sync_committee: FixedBytes::from_slice(
                committee_proof
                    .next_sync_committee
                    .aggregate_pubkey
                    .as_serialized(),
            ),
            committee_keys_root: FixedBytes::from_slice(committee_keys_root.as_slice()),
        }
    }
}

impl SyncCommitteeValidatorPubs {
    /// Computes the committee hash used throughout the project
    ///
    /// # Returns
    /// * `FixedBytes<32>` - Hash identifying the committee
    pub fn get_committee_hash(&self) -> FixedBytes<32> {
        get_committee_hash(self.aggregate_pub)
    }
}

impl From<Vec<String>> for SyncCommitteeValidatorPubs {
    /// Converts a vector of hex-encoded public key strings into `SyncCommitteeValidatorPubs`.
    ///
    /// # Arguments
    ///
    /// * `validator_pubs` - A vector of hex-encoded public key strings.
    ///
    /// # Returns
    ///
    /// A new `SyncCommitteeValidatorPubs` instance with parsed public keys.
    fn from(validator_pubs: Vec<String>) -> Self {
        let validator_pubs = validator_pubs
            .iter()
            .map(|s| {
                let mut bytes = [0u8; 48];
                let hex_str = s.trim_start_matches("0x");
                hex::decode_to_slice(hex_str, &mut bytes).unwrap();
                G1Affine::from_compressed(&bytes).unwrap()
            })
            .collect::<Vec<_>>();

        // Aggregate all public keys into a single G1Projective point
        let aggregate_pub = validator_pubs
            .iter()
            .fold(G1Projective::identity(), |acc, pubkey| {
                acc.add_mixed(pubkey)
            });
        Self {
            validator_pubs,
            aggregate_pub: aggregate_pub.into(),
        }
    }
}

/// Possible errors that can occur during sync committee operations
#[derive(Debug, Error)]
pub enum SyncCommitteeError {
    /// Error communicating with beacon node
    #[error("Beacon error: {0}")]
    Beacon(#[from] crate::clients::beacon_chain::BeaconError),
    /// Error processing beacon state proof
    #[error("Beacon state proof error")]
    BeaconStateProof(beacon_state_proof::error::Error),
}

impl From<beacon_state_proof::error::Error> for SyncCommitteeError {
    fn from(error: beacon_state_proof::error::Error) -> Self {
        SyncCommitteeError::BeaconStateProof(error)
    }
}
