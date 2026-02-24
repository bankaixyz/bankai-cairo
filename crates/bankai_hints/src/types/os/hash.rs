use alloy_primitives::keccak256;
use cairo_vm_base::types::{felt::Felt, uint256::Uint256};
use num_bigint::BigUint;

use super::BankaiBlockCairo;

const BANKAI_BLOCK_HASH_WORDS: usize = 24;

fn felt_to_uint256(value: &Felt) -> Uint256 {
    let value_big = BigUint::from_bytes_be(&value.0.to_bytes_be());
    let mask = (BigUint::from(1u8) << 128usize) - BigUint::from(1u8);
    let low = &value_big & &mask;
    let high = &value_big >> 128usize;
    Uint256((high << 128usize) | low)
}

fn uint256_to_be_word(value: &Uint256) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = value.0.to_bytes_be();
    out[(32 - bytes.len())..32].copy_from_slice(&bytes);
    out
}

pub fn compute_block_hash_keccak(block: &BankaiBlockCairo) -> Uint256 {
    let beacon = &block.ethereum.beacon;
    let execution = &block.ethereum.execution;

    let words: [Uint256; BANKAI_BLOCK_HASH_WORDS] = [
        felt_to_uint256(&block.version),
        felt_to_uint256(&block.program_hash),
        block.prev_block_hash.clone(),
        felt_to_uint256(&block.bankai_mmr_root_poseidon),
        block.bankai_mmr_root_keccak.clone(),
        felt_to_uint256(&block.block_number),
        felt_to_uint256(&beacon.slot_number),
        beacon.header_root.clone(),
        beacon.state_root.clone(),
        felt_to_uint256(&beacon.justified_height),
        felt_to_uint256(&beacon.finalized_height),
        felt_to_uint256(&beacon.num_signers),
        beacon.mmr_root_keccak.clone(),
        felt_to_uint256(&beacon.mmr_root_poseidon),
        felt_to_uint256(&beacon.current_validator_root),
        felt_to_uint256(&beacon.next_validator_root),
        felt_to_uint256(&execution.block_number),
        execution.header_hash.clone(),
        felt_to_uint256(&execution.justified_height),
        felt_to_uint256(&execution.finalized_height),
        execution.mmr_root_keccak.clone(),
        felt_to_uint256(&execution.mmr_root_poseidon),
        block.op_stack.root.clone(),
        felt_to_uint256(&block.op_stack.n_clients),
    ];

    let mut preimage = Vec::with_capacity(BANKAI_BLOCK_HASH_WORDS * 32);
    for word in &words {
        preimage.extend_from_slice(&uint256_to_be_word(word));
    }

    Uint256(BigUint::from_bytes_be(keccak256(preimage).as_slice()))
}