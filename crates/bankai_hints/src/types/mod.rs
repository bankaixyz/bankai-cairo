pub mod bls;
pub mod header;

use cairo_vm_base::cairo_type::{CairoType, CairoWritable};
use cairo_vm_base::types::{
    felt::Felt, uint256::Uint256, uint256_32::Uint256Bits32, uint384::UInt384,
};
use cairo_vm_base::vm::cairo_vm::Felt252;
use serde::{Deserialize, Serialize};

use crate::types::bls::{G1PointCairo, G2PointCairo};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoneCircuitLayoutCairo {
    pub input: StoneInputsCairo,
    pub output: CircuitOutputCairo2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoneInputsCairo {
    pub consensus_data: ConsensusInputsCairo,
    pub sync_committee_update: Option<SyncCommitteeUpdateProofCairo>,
    pub proof_data: Option<ProofDataCairo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofDataCairo {
    pub block_number: u64,
    pub proof: serde_json::Value,
    pub proof_output: CircuitOutputCairo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitOutputCairo2 {
    pub block_number: Felt,
    pub beacon: BeaconClientOutputCairo,
    pub execution: ExecutionClientOutputCairo,
}

impl CairoWritable for CircuitOutputCairo2 {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.block_number.to_memory(vm, current_ptr)?;
        current_ptr = self.beacon.to_memory(vm, current_ptr)?;
        current_ptr = self.execution.to_memory(vm, current_ptr)?;

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Felt::n_fields() + BeaconClientOutputCairo::n_fields() + ExecutionClientOutputCairo::n_fields()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconClientOutputCairo {
    pub slot_number: Felt,
    pub header_root: Uint256,
    pub state_root: Uint256,
    pub justified_height: Felt,
    pub finalized_height: Felt,
    pub num_signers: Felt,
    pub mmr_root_sha: Uint256,
    pub mmr_root_poseidon: Felt,
    pub current_committee_hash: Uint256,
    pub next_committee_hash: Uint256,
}

impl CairoWritable for BeaconClientOutputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.slot_number.to_memory(vm, current_ptr)?;
        current_ptr = self.header_root.to_memory(vm, current_ptr)?;
        current_ptr = self.state_root.to_memory(vm, current_ptr)?;
        current_ptr = self.justified_height.to_memory(vm, current_ptr)?;
        current_ptr = self.finalized_height.to_memory(vm, current_ptr)?;
        current_ptr = self.num_signers.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_sha.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_poseidon.to_memory(vm, current_ptr)?;
        current_ptr = self.current_committee_hash.to_memory(vm, current_ptr)?;
        current_ptr = self.next_committee_hash.to_memory(vm, current_ptr)?;

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Felt::n_fields() * 5 + Uint256::n_fields() * 5
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionClientOutputCairo {
    pub block_number: Felt,
    pub header_hash: Uint256,
    pub justified_height: Felt,
    pub finalized_height: Felt,
    pub mmr_root_sha: Uint256,
    pub mmr_root_poseidon: Felt,
}

impl CairoWritable for ExecutionClientOutputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.block_number.to_memory(vm, current_ptr)?;
        current_ptr = self.header_hash.to_memory(vm, current_ptr)?;
        current_ptr = self.justified_height.to_memory(vm, current_ptr)?;
        current_ptr = self.finalized_height.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_sha.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_poseidon.to_memory(vm, current_ptr)?;

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Felt::n_fields() * 2 + Uint256::n_fields() * 4
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitOutputCairo {
    pub beacon_header_root: Uint256,
    pub beacon_state_root: Uint256,
    pub beacon_height: Felt,
    pub n_signers: Felt,
    pub execution_header_root: Uint256,
    pub execution_header_height: Felt,
    pub current_committee_hash: Uint256,
    pub next_committee_hash: Uint256,
}

impl CairoWritable for CircuitOutputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;

        current_ptr = self.beacon_header_root.to_memory(vm, current_ptr)?;
        current_ptr = self.beacon_state_root.to_memory(vm, current_ptr)?;
        current_ptr = self.beacon_height.to_memory(vm, current_ptr)?;
        current_ptr = self.n_signers.to_memory(vm, current_ptr)?;
        current_ptr = self.execution_header_root.to_memory(vm, current_ptr)?;
        current_ptr = self.execution_header_height.to_memory(vm, current_ptr)?;
        current_ptr = self.current_committee_hash.to_memory(vm, current_ptr)?;
        current_ptr = self.next_committee_hash.to_memory(vm, current_ptr)?;

        // Check that the memory layout is correct
        let expected_ptr = (address + CircuitOutputCairo::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!("Memory layout mismatch for CircuitOutputCairo: expected pointer at {expected_ptr}, but got {current_ptr}").into()
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Uint256::n_fields() * 5 + Felt::n_fields() * 3
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncCommitteeSignerInputCairo {
    pub signature_point: G2PointCairo,
    pub aggregate_pub: G1PointCairo,
    pub non_signers: Vec<G1PointCairo>,
}

impl CairoWritable for SyncCommitteeSignerInputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;

        current_ptr = self.signature_point.to_memory(vm, current_ptr)?;
        current_ptr = self.aggregate_pub.to_memory(vm, current_ptr)?;

        // Create segment for non-signers and store its pointer
        let non_signers_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, non_signers_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write all non-signers to the segment
        let mut segment_ptr = non_signers_segment;
        for non_signer in &self.non_signers {
            segment_ptr = non_signer.to_memory(vm, segment_ptr)?;
        }

        // Store the length of non-signers
        vm.insert_value(current_ptr, Felt252::from(self.non_signers.len() as u64))?;
        current_ptr = (current_ptr + 1)?;

        // Check that the memory layout is correct
        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!("Memory layout mismatch for SignerDataCairo: expected pointer at {expected_ptr}, but got {current_ptr}").into()
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        G1PointCairo::n_fields() + G2PointCairo::n_fields() + 2
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionHeaderProofCairo {
    pub root: Uint256,
    pub path: Vec<Uint256Bits32>,
    pub leaf: Uint256,
    pub index: Felt,
    pub execution_payload_header: Vec<Uint256>,
}

impl CairoWritable for ExecutionHeaderProofCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        println!("ExecutionHeaderProofCairo: {self:?}");
        let mut current_ptr = address;

        current_ptr = self.root.to_memory(vm, current_ptr)?;

        // Create segment for path and store its pointer
        let path_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, path_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write each path element
        let mut segment_ptr = path_segment;
        for path_element in &self.path {
            segment_ptr = path_element.to_memory(vm, segment_ptr)?;
        }

        current_ptr = self.leaf.to_memory(vm, current_ptr)?;
        current_ptr = self.index.to_memory(vm, current_ptr)?;
        println!("writing payload fields");
        // Create segment for payload fields and store its pointer
        let payload_fields_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, payload_fields_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write each payload field
        let mut payload_fields_ptr = payload_fields_segment;
        for field in &self.execution_payload_header {
            payload_fields_ptr = field.to_memory(vm, payload_fields_ptr)?;
        }
        println!("payload fields written");

        // Check that the memory layout is correct
        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!("Memory layout mismatch for ExecutionHeaderProofCairo: expected pointer at {expected_ptr}, but got {current_ptr}").into()
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Uint256::n_fields() + 1 + Uint256::n_fields() + 1 + 1
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconHeaderCairo {
    pub slot: Uint256,
    pub proposer_index: Uint256,
    pub parent_root: Uint256,
    pub state_root: Uint256,
    pub body_root: Uint256,
}

impl CairoWritable for BeaconHeaderCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;

        current_ptr = self.slot.to_memory(vm, current_ptr)?;
        current_ptr = self.proposer_index.to_memory(vm, current_ptr)?;
        current_ptr = self.parent_root.to_memory(vm, current_ptr)?;
        current_ptr = self.state_root.to_memory(vm, current_ptr)?;
        current_ptr = self.body_root.to_memory(vm, current_ptr)?;

        // Check that the memory layout is correct
        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!("Memory layout mismatch for BeaconHeaderCairo: expected pointer at {expected_ptr}, but got {current_ptr}").into()
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Uint256::n_fields() * 5
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusInputsCairo {
    pub beacon_header: BeaconHeaderCairo,
    pub signature: SyncCommitteeSignerInputCairo,
    pub execution_header_proof: ExecutionHeaderProofCairo,
}

impl CairoWritable for ConsensusInputsCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;

        current_ptr = self.beacon_header.to_memory(vm, current_ptr)?;
        current_ptr = self.signature.to_memory(vm, current_ptr)?;
        current_ptr = self.execution_header_proof.to_memory(vm, current_ptr)?;

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        BeaconHeaderCairo::n_fields()
            + SyncCommitteeSignerInputCairo::n_fields()
            + ExecutionHeaderProofCairo::n_fields()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncCommitteeUpdateProofCairo {
    pub beacon_slot: Felt,
    pub next_sync_committee_branch: Vec<Uint256Bits32>,
    pub next_aggregate_sync_committee: UInt384,
    pub committee_keys_root: Uint256Bits32,
}

impl CairoWritable for SyncCommitteeUpdateProofCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;

        current_ptr = self.beacon_slot.to_memory(vm, current_ptr)?;

        // Create segment for next sync committee branch and store its pointer
        let next_sync_committee_branch_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, next_sync_committee_branch_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write each next sync committee branch element
        let mut segment_ptr = next_sync_committee_branch_segment;
        for branch in &self.next_sync_committee_branch {
            segment_ptr = branch.to_memory(vm, segment_ptr)?;
        }

        current_ptr = self
            .next_aggregate_sync_committee
            .to_memory(vm, current_ptr)?;
        current_ptr = self.committee_keys_root.to_memory(vm, current_ptr)?;

        // Check that the memory layout is correct
        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!("Memory layout mismatch for SyncCommitteeDataCairo: expected pointer at {expected_ptr}, but got {current_ptr}").into()
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Felt::n_fields() + 1 + UInt384::n_fields() + Uint256Bits32::n_fields()
    }
}
