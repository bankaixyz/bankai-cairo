use std::collections::HashMap;

use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_ptr_from_var_name;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_relocatable_from_var_name;
use cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm_base::vm::cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm_base::vm::cairo_vm::Felt252;
use cairo_vm_base::cairo_type::CairoWritable;
use cairo_vm_base::cairo_type::CairoType;

use crate::types::StoneInputsCairo;

pub const HINT_WRITE_CONSENSUS_INPUTS: &str = r#"write_consensus_inputs()"#;
pub const HINT_WRITE_STONE_PROOF_INPUTS: &str = r#"write_stone_proof_inputs()"#;
pub const HINT_WRITE_COMMITTEE_UPDATE_INPUTS: &str = r#"write_committee_update_inputs()"#;
pub const HINT_WRITE_EXPECTED_PROOF_OUTPUT: &str = r#"load_expected_proof_output()"#;

pub fn write_consensus_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("inputs")?;
    let epoch_update = &inputs.consensus_inputs;
    let epoch_update_ptr = get_relocatable_from_var_name(
        "consensus_inputs",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    epoch_update.to_memory(vm, epoch_update_ptr)?;

    let is_genesis_ptr = get_relocatable_from_var_name(
        "is_genesis",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    
    let is_genesis = match &inputs.proof_data.proof {
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
    let is_committee_update = match &inputs.sync_committee_update {
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
    // let program_hash = Felt252::from_hex_unchecked(
    //     "0x5b6ff167e72599c14a2e99cac4a6e8db3036db0f0d9acac15d5822ea315287a",
    // );
    let program_hash = Felt252::from_hex_unchecked(
        "0x6305ea579daa2cd35f92ce5c41fa3467a7b44c4d69f9849844aff9d552620e",
    );
    // let program_hash = Felt252::from_hex_unchecked(
    //     "0x5ab580b04e3532b6b18f81cfa654a05e29dd8e2352d88df1e765a84072db07",
    // );
    vm.insert_value(program_hash_ptr, program_hash)?;

    Ok(())
}

pub fn write_expected_proof_output(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("inputs")?;
    if let Some(stark_proof_output) = &inputs.proof_data.proof_output {
        let expected_output_ptr = get_relocatable_from_var_name(
            "expected_proof_output",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;

        stark_proof_output.to_memory(vm, expected_output_ptr)?;
    }

    Ok(())
}

pub fn write_stone_proof_inputs(
    _vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    _hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("inputs")?;
    if let Some(stone_proof) = &inputs.proof_data.proof {
        let proof_string = serde_json::json!({
            "proof": stone_proof
        })
        .to_string();
        exec_scopes.insert_value("program_input", proof_string);
    } else {
        panic!("Stone proof not found");
    }

    Ok(())
}

pub fn write_committee_update_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("inputs")?;
    if let Some(sync_committee_update) = &inputs.sync_committee_update {
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
