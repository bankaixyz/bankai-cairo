use std::collections::HashMap;

use crate::{
    hint_processor::CustomHintProcessor,
    types::{Bytes32, Felt, G1PointCairo, G2PointCairo, UInt384, Uint256, Uint256Bits32},
};
use beacon_types::TreeHash;
use beacon_types::{ExecutionPayloadHeader, MainnetEthSpec};
use cairo_vm::{
    hint_processor::builtin_hint_processor::{
        builtin_hint_processor_definition::HintProcessorData,
        hint_utils::{get_ptr_from_var_name, get_relocatable_from_var_name},
    },
    types::{exec_scope::ExecutionScopes, relocatable::Relocatable},
    vm::{errors::hint_errors::HintError, vm_core::VirtualMachine},
    Felt252,
};
use garaga_zero::types::CairoType;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct RecursiveEpochUpdateCairo {
    pub inputs: RecursiveEpochInputsCairo,
    pub outputs: RecursiveEpochOutputsCairo,
}

#[derive(Debug, Deserialize)]
pub struct RecursiveEpochOutputsCairo {
    pub beacon_header_root: Uint256,
    pub beacon_state_root: Uint256,
    pub beacon_height: Felt,
    pub n_signers: Felt,
    pub execution_header_root: Uint256,
    pub execution_header_height: Felt,
    pub current_committee_hash: Uint256,
    pub next_committee_hash: Uint256,
}

#[derive(Debug, Deserialize)]
pub struct RecursiveEpochInputsCairo {
    pub epoch_update: EpochUpdateCairo,
    pub sync_committee_update: Option<SyncCommitteeDataCairo>,
    pub stark_proof: Option<Value>, // this is the stark proof of the previous epoch update
    pub stark_proof_output: Option<RecursiveEpochOutputsCairo>,
}

#[derive(Debug, Deserialize)]
pub struct EpochUpdateCairo {
    pub header: BeaconHeaderCairo,
    pub signature_point: G2PointCairo,
    pub aggregate_pub: G1PointCairo,
    pub non_signers: Vec<G1PointCairo>,
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
pub struct ExecutionHeaderProofCairo {
    pub root: Uint256,
    pub path: Vec<Uint256Bits32>,
    pub leaf: Uint256,
    pub index: Felt,
    pub execution_payload_header: Vec<Bytes32>,
}

pub struct ExecutionPayloadHeaderCairo(pub ExecutionPayloadHeader<MainnetEthSpec>);
impl ExecutionPayloadHeaderCairo {
    pub fn to_field_roots(&self) -> Vec<Bytes32> {
        // Helper function to convert any value to a padded 32-byte Uint256
        fn to_uint256<T: AsRef<[u8]>>(bytes: T) -> Bytes32 {
            let mut padded = vec![0; 32];
            let bytes = bytes.as_ref();
            // Copy bytes to the beginning of the padded array (right padding with zeros)
            padded[..bytes.len()].copy_from_slice(bytes);
            Bytes32::new(padded)
        }

        // Convert u64 to padded bytes
        fn u64_to_uint256(value: u64) -> Bytes32 {
            Bytes32::from_u64(value)
        }

        macro_rules! extract_common_fields {
            ($h:expr) => {
                vec![
                    to_uint256($h.parent_hash.0.as_slice()),
                    to_uint256($h.fee_recipient.0.to_vec()),
                    to_uint256($h.state_root.0.to_vec()),
                    to_uint256($h.receipts_root.0.to_vec()),
                    to_uint256($h.logs_bloom.tree_hash_root().as_slice()),
                    to_uint256($h.prev_randao.0.to_vec()),
                    u64_to_uint256($h.block_number),
                    u64_to_uint256($h.gas_limit),
                    u64_to_uint256($h.gas_used),
                    u64_to_uint256($h.timestamp),
                    to_uint256($h.extra_data.tree_hash_root().as_slice()),
                    to_uint256($h.base_fee_per_gas.tree_hash_root().as_slice()),
                    to_uint256($h.block_hash.0.as_slice()),
                    to_uint256($h.transactions_root.as_slice()),
                ]
            };
        }

        let roots = match &self.0 {
            ExecutionPayloadHeader::Bellatrix(h) => extract_common_fields!(h),
            ExecutionPayloadHeader::Capella(h) => {
                let mut roots = extract_common_fields!(h);
                roots.push(to_uint256(h.withdrawals_root.as_slice()));
                roots
            }
            ExecutionPayloadHeader::Deneb(h) => {
                let mut roots = extract_common_fields!(h);
                roots.push(to_uint256(h.withdrawals_root.as_slice()));
                roots.push(u64_to_uint256(h.blob_gas_used));
                roots.push(u64_to_uint256(h.excess_blob_gas));
                roots
            }
            ExecutionPayloadHeader::Electra(h) => {
                // The execution payload is the same as Deneb
                let mut roots = extract_common_fields!(h);
                roots.push(to_uint256(h.withdrawals_root.as_slice()));
                roots.push(u64_to_uint256(h.blob_gas_used));
                roots.push(u64_to_uint256(h.excess_blob_gas));
                roots
            }
            ExecutionPayloadHeader::Fulu(_h) => panic!("Fulu not supported"),
        };

        roots
    }
}

#[derive(Debug, Deserialize)]
pub struct SyncCommitteeDataCairo {
    pub beacon_slot: Felt,
    pub next_sync_committee_branch: Vec<Uint256Bits32>,
    pub next_aggregate_sync_committee: UInt384,
    pub committee_keys_root: Uint256Bits32,
}

pub const HINT_WRITE_EPOCH_UPDATE_INPUTS: &str = r#"write_epoch_update_inputs()"#;
pub const HINT_WRITE_STARK_PROOF_INPUTS: &str = r#"write_stark_proof_inputs()"#;
pub const HINT_WRITE_COMMITTEE_UPDATE_INPUTS: &str = r#"write_committee_update_inputs()"#;
pub const HINT_WRITE_EXPECTED_PROOF_OUTPUT: &str = r#"load_expected_proof_output()"#;

impl CustomHintProcessor {
    pub fn write_epoch_update_inputs(
        &self,
        vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        hint_data: &HintProcessorData,
        _constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        let epoch_update = &self.recursive_epoch_update.inputs.epoch_update;
        let epoch_update_ptr = get_relocatable_from_var_name(
            "epoch_update",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        write_epoch_update(epoch_update_ptr, epoch_update, vm)?;

        let is_genesis_ptr = get_relocatable_from_var_name(
            "is_genesis",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        let is_genesis = match &self.recursive_epoch_update.inputs.stark_proof {
            Some(_) => 0,
            None => 1,
        };
        vm.insert_value(is_genesis_ptr, Felt252::from(is_genesis))?;

        let is_committee_update_ptr = get_relocatable_from_var_name(
            "is_committee_update",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        let is_committee_update = match &self.recursive_epoch_update.inputs.sync_committee_update {
            Some(_) => 1,
            None => 0,
        };
        vm.insert_value(is_committee_update_ptr, Felt252::from(is_committee_update))?;

        let program_hash_ptr = get_relocatable_from_var_name(
            "program_hash",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        let program_hash = Felt252::from_hex_unchecked(
            "0x5b6ff167e72599c14a2e99cac4a6e8db3036db0f0d9acac15d5822ea315287a",
        );
        vm.insert_value(program_hash_ptr, program_hash)?;

        Ok(())
    }

    pub fn write_expected_proof_output(
        &self,
        vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        hint_data: &HintProcessorData,
        _constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        let expected_output_ptr = get_relocatable_from_var_name(
            "expected_proof_output",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;

        // Now write the struct data to the new segment
        let values = &self
            .recursive_epoch_update
            .inputs
            .stark_proof_output
            .as_ref()
            .unwrap();

        let mut current_ptr = expected_output_ptr;
        current_ptr = values.beacon_header_root.to_memory(vm, current_ptr)?;
        current_ptr = values.beacon_state_root.to_memory(vm, current_ptr)?;
        current_ptr = values.beacon_height.to_memory(vm, current_ptr)?;
        current_ptr = values.n_signers.to_memory(vm, current_ptr)?;
        current_ptr = values.execution_header_root.to_memory(vm, current_ptr)?;
        current_ptr = values.execution_header_height.to_memory(vm, current_ptr)?;
        current_ptr = values.current_committee_hash.to_memory(vm, current_ptr)?;
        let _current_ptr = values.next_committee_hash.to_memory(vm, current_ptr)?;

        Ok(())
    }

    pub fn write_stark_proof_inputs(
        &self,
        _vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        _hint_data: &HintProcessorData,
        _constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        if let Some(stark_proof) = &self.recursive_epoch_update.inputs.stark_proof {
            let proof_string = serde_json::json!({
                "proof": stark_proof
            })
            .to_string();
            exec_scopes.insert_value("program_input", proof_string);
        } else {
            panic!("Stark proof not found");
        }

        Ok(())
    }

    pub fn write_committee_update_inputs(
        &self,
        vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        hint_data: &HintProcessorData,
        _constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        if let Some(sync_committee_update) =
            &self.recursive_epoch_update.inputs.sync_committee_update
        {
            let aggregate_committee_key_ptr = get_relocatable_from_var_name(
                "aggregate_committee_key",
                vm,
                &hint_data.ids_data,
                &hint_data.ap_tracking,
            )?;
            sync_committee_update
                .next_aggregate_sync_committee
                .to_memory(vm, aggregate_committee_key_ptr)?;

            let committee_keys_root_ptr = get_ptr_from_var_name(
                "committee_keys_root",
                vm,
                &hint_data.ids_data,
                &hint_data.ap_tracking,
            )?;
            sync_committee_update
                .committee_keys_root
                .to_memory(vm, committee_keys_root_ptr)?;

            let path_ptr =
                get_ptr_from_var_name("path", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;

            for (i, branch) in sync_committee_update
                .next_sync_committee_branch
                .iter()
                .enumerate()
            {
                let branch_segment = vm.add_memory_segment();
                branch.to_memory(vm, branch_segment)?;
                vm.insert_value((path_ptr + i)?, branch_segment)?;
            }

            let path_len_ptr = get_relocatable_from_var_name(
                "path_len",
                vm,
                &hint_data.ids_data,
                &hint_data.ap_tracking,
            )?;
            let path_len = Felt252::from(sync_committee_update.next_sync_committee_branch.len());
            vm.insert_value(path_len_ptr, path_len)?;

            Ok(())
        } else {
            panic!("Committee input not found");
        }
    }
}

pub fn write_epoch_update(
    epoch_update_ptr: Relocatable,
    circuit_inputs: &EpochUpdateCairo,
    vm: &mut VirtualMachine,
) -> Result<Relocatable, HintError> {
    let mut current_ptr = epoch_update_ptr;

    // Write signature point
    current_ptr = circuit_inputs.signature_point.to_memory(vm, current_ptr)?;

    // Write header fields
    current_ptr = write_header_fields(vm, current_ptr, &circuit_inputs.header)?;

    // Write signer data (aggregate pub key and non-signers)
    current_ptr = write_signer_data(vm, current_ptr, circuit_inputs)?;

    // Write execution header proof
    current_ptr =
        write_execution_header_proof(vm, current_ptr, &circuit_inputs.execution_header_proof)?;

    Ok(current_ptr)
}

fn write_header_fields(
    vm: &mut VirtualMachine,
    mut ptr: Relocatable,
    header: &BeaconHeaderCairo,
) -> Result<Relocatable, HintError> {
    ptr = header.slot.to_memory(vm, ptr)?;
    ptr = header.proposer_index.to_memory(vm, ptr)?;
    ptr = header.parent_root.to_memory(vm, ptr)?;
    ptr = header.state_root.to_memory(vm, ptr)?;
    ptr = header.body_root.to_memory(vm, ptr)?;
    Ok(ptr)
}

fn write_signer_data(
    vm: &mut VirtualMachine,
    mut ptr: Relocatable,
    circuit_inputs: &EpochUpdateCairo,
) -> Result<Relocatable, HintError> {
    // Write aggregate public key
    ptr = circuit_inputs.aggregate_pub.to_memory(vm, ptr)?;

    // Create segment for non-signers and store its pointer
    let non_signers_segment = vm.add_memory_segment();
    vm.insert_value(ptr, non_signers_segment)?;

    // Write all non-signers to the segment
    let mut segment_ptr = non_signers_segment;
    for non_signer in &circuit_inputs.non_signers {
        segment_ptr = non_signer.to_memory(vm, segment_ptr)?;
    }

    // Store the length of non-signers
    vm.insert_value((ptr + 1)?, Felt252::from(circuit_inputs.non_signers.len()))?;

    Ok((ptr + 2)?)
}

fn write_execution_header_proof(
    vm: &mut VirtualMachine,
    mut ptr: Relocatable,
    proof: &ExecutionHeaderProofCairo,
) -> Result<Relocatable, HintError> {
    // Write root
    ptr = proof.root.to_memory(vm, ptr)?;

    // Create and write path segment
    let path_segment = vm.add_memory_segment();
    vm.insert_value(ptr, path_segment)?;
    ptr = (ptr + 1)?;

    // Write each path element
    let mut path_ptr = path_segment;
    for path_element in &proof.path {
        let element_segment = vm.add_memory_segment();
        vm.insert_value(path_ptr, element_segment)?;
        path_ptr = (path_ptr + 1)?;

        path_element.to_memory(vm, element_segment)?;
    }

    // Write leaf and index
    ptr = proof.leaf.to_memory(vm, ptr)?;
    ptr = proof.index.to_memory(vm, ptr)?;

    // Create and write payload fields segment
    let payload_fields_segment = vm.add_memory_segment();
    vm.insert_value(ptr, payload_fields_segment)?;

    // Write each payload field
    let mut payload_fields_ptr = payload_fields_segment;
    for field in &proof.execution_payload_header {
        payload_fields_ptr = field.to_memory(vm, payload_fields_ptr)?;
    }

    Ok((ptr + 1)?)
}
