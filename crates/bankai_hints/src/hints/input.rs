use std::collections::HashMap;

use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_relocatable_from_var_name;
use cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable;
use cairo_vm_base::vm::cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm_base::vm::cairo_vm::Felt252;
use cairo_vm_base::cairo_type::CairoWritable;

use crate::types::RecursiveEpochInputsCairo;


pub fn write_epoch_update_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<RecursiveEpochInputsCairo>("inputs")?;
    let epoch_update = &inputs.epoch_update;
    let epoch_update_ptr = get_relocatable_from_var_name(
        "epoch_update",
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
    let is_genesis = match &inputs.stark_proof {
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
        "0x5b6ff167e72599c14a2e99cac4a6e8db3036db0f0d9acac15d5822ea315287a",
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
    let inputs = exec_scopes.get_ref::<RecursiveEpochInputsCairo>("inputs")?;
    if let Some(stark_proof_output) = &inputs.stark_proof_output {

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