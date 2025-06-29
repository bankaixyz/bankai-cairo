use std::collections::HashMap;

use cairo_vm::{
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
use garaga_zero::types::CairoType;

use crate::types::Uint256;

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

pub const PRINT_FELT_HEX: &str = "print(f\"{hex(ids.value)}\")";

pub fn print_felt_hex(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let value =
        get_integer_from_var_name("value", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    println!("Value: {}", value.to_hex_string());
    Ok(())
}

pub const PRINT_FELT: &str = "print(f\"{ids.value}\")";

pub fn print_felt(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let value =
        get_integer_from_var_name("value", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    println!("Value: {}", value);
    Ok(())
}

pub const PRINT_STRING: &str = "print(f\"String: {ids.value}\")";

pub fn print_string(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let value =
        get_integer_from_var_name("value", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    let bytes = value.to_bytes_be();
    let ascii = String::from_utf8_lossy(&bytes);
    println!("String: {}", ascii);
    Ok(())
}

pub const PRINT_UINT256: &str =
    "print(f\"Uint256: {hex(ids.value.low + 128**2 * ids.value.high)}\")";

pub fn print_uint256(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let value = Uint256::from_memory(
        vm,
        get_relocatable_from_var_name("value", vm, &hint_data.ids_data, &hint_data.ap_tracking)?,
    )?;
    println!("Uint256: {}", value.0.to_str_radix(16));
    Ok(())
}
