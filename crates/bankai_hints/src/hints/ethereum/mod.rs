use cairo_vm_base::{
    cairo_type::CairoWritable,
    vm::cairo_vm::{
        hint_processor::builtin_hint_processor::{
            builtin_hint_processor_definition::HintProcessorData,
            hint_utils::{
                get_integer_from_var_name, get_ptr_from_var_name, get_relocatable_from_var_name,
            },
        },
        types::exec_scope::ExecutionScopes,
        vm::{errors::hint_errors::HintError, vm_core::VirtualMachine},
        Felt252,
    },
};
use std::collections::HashMap;

use crate::types::os::BankaiBlockInputsCairo;

pub const HINT_WRITE_CONSENSUS_INPUTS: &str = r#"write_consensus_inputs()"#;
pub const HINT_WRITE_COMMITTEE_UPDATE_INPUTS: &str = r#"write_committee_update_inputs()"#;
pub const HINT_CHECK_FORK_VERSION: &str = r#"check_fork_version()"#;
pub const HINT_WRITE_OP_STACK_INPUTS: &str = r#"write_op_stack_inputs()"#;

pub fn write_consensus_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<BankaiBlockInputsCairo>("input")?;
    let ethereum_inputs = &inputs.ethereum;

    let consensus_data_ptr = get_relocatable_from_var_name(
        "consensus_inputs",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    ethereum_inputs
        .consensus_data
        .to_memory(vm, consensus_data_ptr)?;

    let is_committee_update_ptr = get_relocatable_from_var_name(
        "is_committee_update",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    let is_committee_update = match &ethereum_inputs.sync_committee_update {
        Some(_) => 1,
        None => 0,
    };
    vm.insert_value(is_committee_update_ptr, Felt252::from(is_committee_update))?;

    Ok(())
}

pub fn write_committee_update_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<BankaiBlockInputsCairo>("input")?;

    if let Some(sync_committee_update) = &inputs.ethereum.sync_committee_update {
        let committee_input_ptr = get_relocatable_from_var_name(
            "committee_input",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        sync_committee_update.to_memory(vm, committee_input_ptr)?;
    } else {
        panic!("Committee update not found");
    }

    Ok(())
}

pub fn hint_check_fork_version(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let slot = get_integer_from_var_name("slot", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;

    // Get the fork_data label address from Cairo memory
    let fork_schedule_ptr = get_ptr_from_var_name(
        "fork_schedule",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    let n_hardforks: usize = get_integer_from_var_name(
        "n_hardforks",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?
    .try_into()
    .unwrap();

    // Read activation slots for the selected network
    let mut activation_slots = Vec::new();
    for i in 0..n_hardforks {
        let slot_address = (fork_schedule_ptr + (i * 2 + 1))?;
        let activation_slot = *vm.get_integer(slot_address)?;
        activation_slots.push(activation_slot);
    }

    let mut latest_fork = 0;
    for (i, activation_slot) in activation_slots.iter().enumerate() {
        if slot >= *activation_slot {
            latest_fork = i;
        }
    }

    // Store the fork value in the Cairo program
    let fork =
        get_relocatable_from_var_name("fork", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    vm.insert_value(fork, Felt252::from(latest_fork))?;

    Ok(())
}

pub fn write_op_stack_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<BankaiBlockInputsCairo>("input")?;
    let op_stack_inputs = &inputs.op_stack;

    let op_stack_inputs_ptr = get_relocatable_from_var_name(
        "op_inputs",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    op_stack_inputs.to_memory(vm, op_stack_inputs_ptr)?;

    Ok(())
}
