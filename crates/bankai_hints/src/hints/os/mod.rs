use std::collections::HashMap;

use cairo_vm_base::{cairo_type::CairoType, types::felt::Felt};
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_relocatable_from_var_name;
use cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm_base::vm::cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm_base::vm::cairo_vm::Felt252;

use crate::types::os::{BankaiBlockCairo, BankaiBlockInputsCairo};

pub const HINT_WRITE_INIT_DATA: &str = r#"write_init_data()"#;
pub const HINT_PREVIOUS_BLOCK: &str = r#"write_previous_block()"#;
pub const HINT_WRITE_MOCK_RECURSION_INPUTS: &str = r#"write_mock_recursion_inputs()"#;
pub const HINT_VERIFY_BLOCK_RESULT: &str = r#"verify_block_result()"#;

pub fn write_init_data(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<BankaiBlockInputsCairo>("input")?;

    let is_genesis_ptr = get_relocatable_from_var_name(
        "is_genesis",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    let is_genesis = if inputs.prev.block_number == Felt(Felt252::from(0)) {
        1
    } else {
        0
    };

    vm.insert_value(is_genesis_ptr, Felt252::from(is_genesis))?;

    let program_hash_ptr = get_relocatable_from_var_name(
        "program_hash",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;

    inputs
        .recursion
        .program_hash
        .to_memory(vm, program_hash_ptr)?;

    Ok(())
}

pub fn write_previous_block(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<BankaiBlockInputsCairo>("input")?;
    let prev = &inputs.prev;
    let prev_ptr =
        get_relocatable_from_var_name("prev", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;

    prev.to_memory(vm, prev_ptr)?;
    Ok(())
}

pub fn write_mock_recursion_inputs(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let inputs = exec_scopes.get_ref::<BankaiBlockInputsCairo>("input")?;
    let recursion = &inputs.recursion;

    let recursion_ptr = get_relocatable_from_var_name(
        "recursion",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    recursion.to_memory(vm, recursion_ptr)?;
    Ok(())
}

pub fn verify_block_result(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let exp_block = exec_scopes.get_ref::<BankaiBlockCairo>("output")?;
    let block_ptr =
        get_relocatable_from_var_name("block", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    let block = BankaiBlockCairo::from_memory(vm, block_ptr)?;

    if block != *exp_block {
        println!("Output mismatch:");
        println!("Block output: {block:#?}");
        println!("Expected block: {exp_block:#?}");
        return Err(HintError::CustomHint(
            format!("Block mismatch: {block:#?} != {exp_block:#?}").into(),
        ));
    }

    Ok(())
}
