pub mod bls;
pub mod header;

use cairo_vm_base::cairo_type::{CairoType, CairoWritable};
use cairo_vm_base::types::{
    felt::Felt, uint256::Uint256, uint256_32::Uint256Bits32, uint384::UInt384,
};
use cairo_vm_base::vm::cairo_vm::Felt252;
use serde::{Deserialize, Serialize};

use crate::types::ethereum::bls::{G1PointCairo, G2PointCairo};
pub use mmr_header_accumulator_hints::types::{
    BeaconHeaderCairo as MmrBeaconHeaderCairo, BeaconMmrUpdateCairo, ExecutionHeaderCairo,
    ExecutionMmrUpdateCairo, LastLeafProofCairo, MmrSnapshotCairo,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EthereumInputsCairo {
    pub consensus_data: ConsensusInputsCairo,
    pub sync_committee_update: Option<SyncCommitteeUpdateProofCairo>,
    pub beacon_mmr_update: BeaconMmrUpdateCairo,
    pub execution_mmr_update: ExecutionMmrUpdateCairo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusInputsCairo {
    pub beacon_header: BeaconHeaderCairo,
    pub signature_point: G2PointCairo,
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
        current_ptr = self.signature_point.to_memory(vm, current_ptr)?;
        current_ptr = self.signature.to_memory(vm, current_ptr)?;
        current_ptr = self.execution_header_proof.to_memory(vm, current_ptr)?;

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        BeaconHeaderCairo::n_fields()
            + G2PointCairo::n_fields()
            + SyncCommitteeSignerInputCairo::n_fields()
            + ExecutionHeaderProofCairo::n_fields()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct EthereumClientOutputCairo {
    pub beacon: BeaconClientOutputCairo,
    pub execution: ExecutionClientOutputCairo,
}

impl CairoType for EthereumClientOutputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.beacon.to_memory(vm, current_ptr)?;
        current_ptr = self.execution.to_memory(vm, current_ptr)?;
        Ok(current_ptr)
    }

    fn from_memory(
        vm: &cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<Self, cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError> {
        Ok(Self {
            beacon: BeaconClientOutputCairo::from_memory(vm, address)?,
            execution: ExecutionClientOutputCairo::from_memory(vm, (address + 1)?)?,
        })
    }

    fn n_fields() -> usize {
        BeaconClientOutputCairo::n_fields() + ExecutionClientOutputCairo::n_fields()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BeaconClientOutputCairo {
    pub slot_number: Felt,
    pub header_root: Uint256,
    pub state_root: Uint256,
    pub justified_height: Felt,
    pub finalized_height: Felt,
    pub num_signers: Felt,
    pub mmr_root_keccak: Uint256,
    pub mmr_root_poseidon: Felt,
    pub current_validator_root: Felt,
    pub next_validator_root: Felt,
}

impl CairoType for BeaconClientOutputCairo {
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
        current_ptr = self.mmr_root_keccak.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_poseidon.to_memory(vm, current_ptr)?;
        current_ptr = self.current_validator_root.to_memory(vm, current_ptr)?;
        current_ptr = self.next_validator_root.to_memory(vm, current_ptr)?;

        Ok(current_ptr)
    }

    fn from_memory(
        vm: &cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<Self, cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError> {
        Ok(Self {
            slot_number: Felt::from_memory(vm, address)?,
            header_root: Uint256::from_memory(vm, (address + 1)?)?,
            state_root: Uint256::from_memory(vm, (address + 3)?)?,
            justified_height: Felt::from_memory(vm, (address + 5)?)?,
            finalized_height: Felt::from_memory(vm, (address + 6)?)?,
            num_signers: Felt::from_memory(vm, (address + 7)?)?,
            mmr_root_keccak: Uint256::from_memory(vm, (address + 8)?)?,
            mmr_root_poseidon: Felt::from_memory(vm, (address + 10)?)?,
            current_validator_root: Felt::from_memory(vm, (address + 11)?)?,
            next_validator_root: Felt::from_memory(vm, (address + 12)?)?,
        })
    }

    fn n_fields() -> usize {
        Felt::n_fields() * 7 + Uint256::n_fields() * 3
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExecutionClientOutputCairo {
    pub block_number: Felt,
    pub header_hash: Uint256,
    pub justified_height: Felt,
    pub finalized_height: Felt,
    pub mmr_root_keccak: Uint256,
    pub mmr_root_poseidon: Felt,
}

impl CairoType for ExecutionClientOutputCairo {
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
        current_ptr = self.mmr_root_keccak.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_poseidon.to_memory(vm, current_ptr)?;

        Ok(current_ptr)
    }

    fn from_memory(
        vm: &cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<Self, cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError> {
        Ok(Self {
            block_number: Felt::from_memory(vm, address)?,
            header_hash: Uint256::from_memory(vm, (address + 1)?)?,
            justified_height: Felt::from_memory(vm, (address + 3)?)?,
            finalized_height: Felt::from_memory(vm, (address + 4)?)?,
            mmr_root_keccak: Uint256::from_memory(vm, (address + 5)?)?,
            mmr_root_poseidon: Felt::from_memory(vm, (address + 7)?)?,
        })
    }

    fn n_fields() -> usize {
        Felt::n_fields() * 4 + Uint256::n_fields() * 2
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncCommitteeSignerInputCairo {
    pub validator_root: Felt,
    pub signers: Vec<G1PointCairo>,
    pub indexes: Vec<Felt>,
    pub proofs: Vec<Vec<Felt>>,
    pub proofs_len: Felt,
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

        current_ptr = self.validator_root.to_memory(vm, current_ptr)?;

        // Create segment for signers and store its pointer
        let signers_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, signers_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write all signers to the segment
        let mut signers_ptr = signers_segment;
        for signer in &self.signers {
            signers_ptr = signer.to_memory(vm, signers_ptr)?;
        }

        // Create segment for indexes and store its pointer
        let indexes_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, indexes_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write all indexes to the segment
        let mut indexes_ptr = indexes_segment;
        for index in &self.indexes {
            indexes_ptr = index.to_memory(vm, indexes_ptr)?;
        }

        // Create segment for proofs (array of pointers) and store its pointer
        let proofs_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, proofs_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write each proof path to its own segment and store pointers
        let mut proofs_ptr = proofs_segment;
        for proof in &self.proofs {
            let proof_segment = vm.add_memory_segment();
            vm.insert_value(proofs_ptr, proof_segment)?;
            proofs_ptr = (proofs_ptr + 1)?;

            let mut proof_segment_ptr = proof_segment;
            for node in proof {
                proof_segment_ptr = node.to_memory(vm, proof_segment_ptr)?;
            }
        }

        current_ptr = self.proofs_len.to_memory(vm, current_ptr)?;
        let n_signers = Felt252::from(self.signers.len() as u64);
        vm.insert_value(current_ptr, n_signers)?;
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
        Felt::n_fields() * 6
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
        let mut current_ptr: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable = address;

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
pub struct SyncCommitteeUpdateProofCairo {
    pub slot: Felt,
    pub path: Vec<Uint256Bits32>,
    pub aggregate_committee_key: UInt384,
    pub validator_pubs: Vec<UInt384>,
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

        current_ptr = self.slot.to_memory(vm, current_ptr)?;
        // Create segment for next sync committee branch and store its pointer
        let path_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, path_segment)?;
        current_ptr = (current_ptr + 1)?;

        // Write each next sync committee branch element
        let mut segment_ptr = path_segment;
        for branch in &self.path {
            segment_ptr = branch.to_memory(vm, segment_ptr)?;
        }

        vm.insert_value(current_ptr, Felt252::from(self.path.len() as u64))?;
        current_ptr = (current_ptr + 1)?;

        current_ptr = self.aggregate_committee_key.to_memory(vm, current_ptr)?;

        // Create segment for validator pubs and store its pointer
        let validator_pubs_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, validator_pubs_segment)?;
        current_ptr = (current_ptr + 1)?;

        let mut validator_pubs_ptr = validator_pubs_segment;
        for pubkey in &self.validator_pubs {
            validator_pubs_ptr = pubkey.to_memory(vm, validator_pubs_ptr)?;
        }

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
        Felt::n_fields() * 5 + UInt384::n_fields()
    }
}
