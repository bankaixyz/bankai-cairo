use std::collections::HashMap;

use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
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
pub const HINT_WRITE_EXPECTED_PROOF_OUTPUT: &str = r#"load_previous_output()"#;

pub fn write_consensus_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("input")?;
    let consensus_data = &inputs.consensus_data;
    let consensus_data_ptr = get_relocatable_from_var_name(
        "consensus_inputs",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    consensus_data.to_memory(vm, consensus_data_ptr)?;

    let is_genesis_ptr = get_relocatable_from_var_name(
        "is_genesis",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    let is_genesis = match &inputs.proof_data {
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
    let program_hash = Felt252::from_hex_unchecked(
        "0x13b99c4febbce601fadb03261d9cea3a81c6e3aecd61002986a9476667870dd",
    );

    vm.insert_value(program_hash_ptr, program_hash)?;

    Ok(())
}

pub fn write_expected_proof_output(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("input")?;
    if let Some(proof_data) = &inputs.proof_data {
        let expected_output_ptr = get_relocatable_from_var_name(
            "previous_output",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;

        proof_data.proof_output.to_memory(vm, expected_output_ptr)?;
    }

    Ok(())
}

pub fn write_stone_proof_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    _hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("input")?;
    let mock_mode_ptr = get_relocatable_from_var_name(
        "mock_mode",
        vm,
        &_hint_data.ids_data,
        &_hint_data.ap_tracking,
    )?;

    if let Some(proof_data) = &inputs.proof_data {
        if proof_data.proof == serde_json::Value::Null {
            vm.insert_value(mock_mode_ptr, Felt252::from(1))?;
        } else {
            let proof_string = serde_json::json!({
                "proof": proof_data.proof
            })
            .to_string();

            vm.insert_value(mock_mode_ptr, Felt252::from(0))?;
            exec_scopes.insert_value("program_input", proof_string);
        }
    } else {
        panic!("Proof data not found");
    }

    Ok(())
}

pub fn write_committee_update_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<StoneInputsCairo>("input")?;
    if let Some(sync_committee_update) = &inputs.sync_committee_update {
        let committee_input_ptr = get_relocatable_from_var_name(
            "committee_input",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;

        sync_committee_update.to_memory(vm, committee_input_ptr)?;

        Ok(())
    } else {
        panic!("Committee input not found");
    }
}
