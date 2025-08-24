use std::collections::HashMap;

use cairo_vm::air_public_input::{MemorySegmentAddresses, PublicMemoryEntry};
use cairo_vm::serde::deserialize_program::Identifier;
use cairo_vm::Felt252;
use serde::{Deserialize, Serialize};

pub(crate) type ProgramIdentifiers = HashMap<String, Identifier>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildProof {
    pub proof: HashMap<String, serde_json::Value>, //serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct CairoVerifierInput {
    pub proof: HashMap<String, serde_json::Value>,
}

pub struct ExtractedProofValues {
    pub original_commitment_hash: Felt252,
    pub interaction_commitment_hash: Felt252,
    pub composition_commitment_hash: Felt252,
    pub oods_values: Vec<Felt252>,
    pub fri_layers_commitments: Vec<Felt252>,
    pub fri_last_layer_coefficients: Vec<Felt252>,
    pub proof_of_work_nonce: Felt252,
    pub original_witness_leaves: Vec<Felt252>,
    pub original_witness_authentications: Vec<Felt252>,
    pub interaction_witness_leaves: Vec<Felt252>,
    pub interaction_witness_authentications: Vec<Felt252>,
    pub composition_witness_leaves: Vec<Felt252>,
    pub composition_witness_authentications: Vec<Felt252>,
    pub fri_step_list: Vec<u64>,
    pub n_fri_layers: usize,
    pub log_n_cosets: u64,
    pub log_last_layer_degree_bound: u32,
    pub n_verifier_friendly_commitment_layers: u64,
    pub z: Felt252,
    pub alpha: Felt252,
    pub proof_of_work_bits: u64,
    pub n_queries: u64,
    pub fri_witnesses_leaves: Vec<Vec<Felt252>>,
    pub fri_witnesses_authentications: Vec<Vec<Felt252>>,
}

#[derive(Debug)]
pub struct ExtractedIDsAndInputValues {
    pub log_trace_domain_size: Felt252,
    pub log_eval_domain_size: Felt252,
    pub layer_log_sizes: Vec<Felt252>,
    pub num_columns_first: Felt252,
    pub num_columns_second: Felt252,
    pub constraint_degree: Felt252,
}

// Struct needed for deserialization of the public input. Owned version of PublicInput.
#[derive(Serialize, Deserialize, Debug)]
pub struct OwnedPublicInput {
    pub layout: String,
    pub rc_min: isize,
    pub rc_max: isize,
    pub n_steps: usize,
    pub memory_segments: HashMap<String, MemorySegmentAddresses>,
    pub public_memory: Vec<PublicMemoryEntry>,
    pub dynamic_params: Option<HashMap<String, u128>>,
}
