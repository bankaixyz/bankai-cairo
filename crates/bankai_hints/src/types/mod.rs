mod bls;
mod header;

use cairo_vm_base::cairo_type::{CairoType, CairoWritable};
use cairo_vm_base::types::{felt::Felt, uint256::Uint256, uint384::UInt384, uint256_32::Uint256Bits32};
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{get_ptr_from_var_name, get_relocatable_from_var_name};
use cairo_vm_base::vm::cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm_base::vm::cairo_vm::Felt252;
use serde::{Deserialize, Serialize};

use crate::types::{bls::{G1PointCairo, G2PointCairo}, header::ExecutionPayloadHeaderCairo};


#[derive(Debug, Deserialize)]
pub struct EpochUpdateCairo {
    pub signature_point: G2PointCairo,
    pub header: BeaconHeaderCairo,
    pub signer_data: SignerDataCairo,
    pub execution_header_proof: ExecutionHeaderProofCairo,
}

#[derive(Debug, Deserialize)]
pub struct BeaconHeaderCairo {
    pub slot: Uint256,
    pub proposer_index: Uint256,
    pub parent_root: Uint256,
    pub state_root: Uint256,
    pub body_root: Uint256,
}

#[derive(Debug, Deserialize)]
pub struct SignerDataCairo {
    pub aggregate_pub: G1PointCairo,
    pub non_signers: Vec<G1PointCairo>,
    pub n_non_signers: Felt,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionHeaderProofCairo {
    pub root: Uint256,
    pub path: Vec<Uint256Bits32>,
    pub leaf: Uint256,
    pub index: Felt,
    pub execution_payload_header: Vec<Uint256>,
}

#[derive(Debug, Deserialize)]
pub struct SyncCommitteeDataCairo {
    pub beacon_slot: Felt,
    pub next_sync_committee_branch: Vec<Uint256Bits32>,
    pub next_aggregate_sync_committee: UInt384,
    pub committee_keys_root: Uint256Bits32,
}
