use std::collections::HashMap;

use cairo_vm_base::vm::cairo_vm::{
    hint_processor::builtin_hint_processor::{
        builtin_hint_processor_definition::HintProcessorData,
        hint_utils::{
            get_integer_from_var_name, get_ptr_from_var_name, get_relocatable_from_var_name,
        },
    },
    types::exec_scope::ExecutionScopes,
    vm::{errors::hint_errors::HintError, vm_core::VirtualMachine},
    Felt252,
};

pub const HINT_CHECK_FORK_VERSION: &str = r#"check_fork_version()"#;

pub fn hint_check_fork_version(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let slot = get_integer_from_var_name("slot", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    let network_id: usize = get_integer_from_var_name(
        "network_id",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?
    .try_into()
    .unwrap();

    // Get the fork_data label address from Cairo memory
    let fork_schedule_ptr = get_ptr_from_var_name(
        "fork_schedule",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    // Each network has 12 values (6 forks × 2 values per fork)
    // For each fork: [version, slot]
    let network_offset = network_id * 12;

    // Read activation slots for the selected network
    let mut activation_slots = Vec::new();
    for i in 0..6 {
        let slot_address = (fork_schedule_ptr + (i * 2 + 1 + network_offset))?;
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

pub const HINT_SET_NEXT_POWER_OF_2: &str = r#"set_next_power_of_2()"#;

pub fn set_next_power_of_2(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let batch_len: usize =
        get_integer_from_var_name("batch_len", vm, &hint_data.ids_data, &hint_data.ap_tracking)?
            .try_into()
            .unwrap();
    let mut next_power_of_2: usize = 1;
    let mut power: usize = 0;
    while next_power_of_2 < batch_len {
        next_power_of_2 *= 2;
        power += 1;
    }
    vm.insert_value(
        get_relocatable_from_var_name(
            "next_power_of_2_index",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?,
        power,
    )?;
    Ok(())
}

pub const HINT_COMPUTE_EPOCH_FROM_SLOT: &str = r#"compute_epoch_from_slot()"#;
pub fn compute_epoch_from_slot(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let current_slot: usize = get_integer_from_var_name(
        "current_slot",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?
    .try_into()
    .unwrap();

    // Calculate current epoch: slot / 32 (integer division automatically floors)
    let current_epoch = current_slot / 32;
    vm.insert_value(
        get_relocatable_from_var_name(
            "current_epoch",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?,
        current_epoch,
    )?;

    Ok(())
}
