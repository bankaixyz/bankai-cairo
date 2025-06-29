use alloy_primitives::FixedBytes;
use serde::{Deserialize, Serialize};
use starknet_core::types::Felt;

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveEpochOutput {
    pub beacon_header_root: FixedBytes<32>,
    pub beacon_state_root: FixedBytes<32>,
    pub beacon_height: u64,
    pub n_signers: u64,
    pub execution_header_root: FixedBytes<32>,
    pub execution_header_height: u64,
    pub current_committee_hash: FixedBytes<32>,
    pub next_committee_hash: FixedBytes<32>,
}

impl RecursiveEpochOutput {
    pub fn from_proof_output(proof_output: Vec<Felt>) -> Result<Self, Error> {
        let program_hash = proof_output[2];
        // Ensure we used the correct Bankai cairo program
        assert_eq!(
            program_hash,
            Felt::from_hex_unchecked(
                "0x5b6ff167e72599c14a2e99cac4a6e8db3036db0f0d9acac15d5822ea315287a"
            )
        );

        let epoch_output = RecursiveEpochOutput {
            beacon_header_root: felts_to_bytes32(&proof_output[3], &proof_output[4]),
            beacon_state_root: felts_to_bytes32(&proof_output[5], &proof_output[6]),
            beacon_height: proof_output[7].try_into().unwrap(),
            n_signers: proof_output[8].try_into().unwrap(),
            execution_header_root: felts_to_bytes32(&proof_output[9], &proof_output[10]),
            execution_header_height: proof_output[11].try_into().unwrap(),
            current_committee_hash: felts_to_bytes32(&proof_output[12], &proof_output[13]),
            next_committee_hash: felts_to_bytes32(&proof_output[14], &proof_output[15]),
        };
        Ok(epoch_output)
    }
}

fn felts_to_bytes32(low: &Felt, high: &Felt) -> FixedBytes<32> {
    let mut bytes = [0u8; 32];
    let low_bytes = low.to_bytes_be();
    let high_bytes = high.to_bytes_be();
    bytes[16..32].copy_from_slice(&low_bytes[16..32]);
    bytes[0..16].copy_from_slice(&high_bytes[16..32]);
    FixedBytes(bytes)
}
