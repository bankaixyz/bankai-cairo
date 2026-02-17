use std::collections::HashMap;

use alloy_primitives::keccak256;
use cairo_vm_base::{cairo_type::CairoType, types::felt::Felt};
use cairo_vm_base::types::uint256::Uint256;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_relocatable_from_var_name;
use cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm_base::vm::cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm_base::vm::cairo_vm::Felt252;
use num_bigint::BigUint;

use crate::types::os::{BankaiBlockCairo, BankaiBlockInputsCairo, BankaiBlockOutputCairo};

pub const HINT_WRITE_INIT_DATA: &str = r#"write_init_data()"#;
pub const HINT_PREVIOUS_BLOCK: &str = r#"write_previous_block()"#;
pub const HINT_WRITE_MOCK_RECURSION_INPUTS: &str = r#"write_mock_recursion_inputs()"#;
pub const HINT_VERIFY_BLOCK_RESULT: &str = r#"verify_block_result()"#;

fn felt_to_u256_word(value: &Felt) -> [u8; 32] {
    let value_big = BigUint::from_bytes_be(&value.0.to_bytes_be());
    let mask = (BigUint::from(1u8) << 128) - BigUint::from(1u8);
    let low: BigUint = &value_big & &mask;
    let high: BigUint = &value_big >> 128usize;

    debug_assert!(low.bits() <= 128);
    debug_assert!(high.bits() <= 128);

    let mut out = [0u8; 32];
    let low_bytes = low.to_bytes_be();
    let high_bytes = high.to_bytes_be();
    out[(16 - high_bytes.len())..16].copy_from_slice(&high_bytes);
    out[(32 - low_bytes.len())..32].copy_from_slice(&low_bytes);
    out
}

fn uint256_to_u256_word(value: &Uint256) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = value.0.to_bytes_be();
    out[(32 - bytes.len())..32].copy_from_slice(&bytes);
    out
}

fn append_felt_as_u256_word(buffer: &mut Vec<u8>, value: &Felt) {
    buffer.extend_from_slice(&felt_to_u256_word(value));
}

fn append_uint256_word(buffer: &mut Vec<u8>, value: &Uint256) {
    buffer.extend_from_slice(&uint256_to_u256_word(value));
}

fn compute_block_hash_keccak(block: &BankaiBlockCairo) -> Uint256 {
    // Bankai block hash input is 22 Uint256 words in strict field order.
    let mut preimage = Vec::with_capacity(22 * 32);

    append_felt_as_u256_word(&mut preimage, &block.version);
    append_felt_as_u256_word(&mut preimage, &block.program_hash);
    append_uint256_word(&mut preimage, &block.prev_block_hash);
    append_felt_as_u256_word(&mut preimage, &block.bankai_mmr_root_poseidon);
    append_uint256_word(&mut preimage, &block.bankai_mmr_root_keccak);
    append_felt_as_u256_word(&mut preimage, &block.block_number);

    append_felt_as_u256_word(&mut preimage, &block.ethereum.beacon.slot_number);
    append_uint256_word(&mut preimage, &block.ethereum.beacon.header_root);
    append_uint256_word(&mut preimage, &block.ethereum.beacon.state_root);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.beacon.justified_height);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.beacon.finalized_height);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.beacon.num_signers);
    append_uint256_word(&mut preimage, &block.ethereum.beacon.mmr_root_keccak);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.beacon.mmr_root_poseidon);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.beacon.current_validator_root);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.beacon.next_validator_root);

    append_felt_as_u256_word(&mut preimage, &block.ethereum.execution.block_number);
    append_uint256_word(&mut preimage, &block.ethereum.execution.header_hash);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.execution.justified_height);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.execution.finalized_height);
    append_uint256_word(&mut preimage, &block.ethereum.execution.mmr_root_keccak);
    append_felt_as_u256_word(&mut preimage, &block.ethereum.execution.mmr_root_poseidon);

    Uint256(BigUint::from_bytes_be(keccak256(preimage).as_slice()))
}

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
    let exp_output = exec_scopes.get_ref::<BankaiBlockOutputCairo>("output")?;
    let exp_block = &exp_output.block;
    let exp_block_hash = &exp_output.block_hash;

    let block_ptr =
        get_relocatable_from_var_name("block", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    let block = BankaiBlockCairo::from_memory(vm, block_ptr)?;
    let block_hash_ptr = get_relocatable_from_var_name(
        "block_hash",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    let block_hash = Uint256::from_memory(vm, block_hash_ptr)?;
    let computed_block_hash = compute_block_hash_keccak(&block);
    let computed_expected_hash = compute_block_hash_keccak(exp_block);

    if block != *exp_block {
        println!("Output mismatch:");
        println!("Block output: {block:#?}");
        println!("Expected block: {exp_block:#?}");
        return Err(HintError::CustomHint(
            format!("Block mismatch: {block:#?} != {exp_block:#?}").into(),
        ));
    }

    if block_hash != *exp_block_hash {
        println!("Block hash output mismatch:");
        println!("Block hash output: {block_hash:#?}");
        println!("Expected block hash: {exp_block_hash:#?}");
        return Err(HintError::CustomHint(
            format!("Block hash mismatch: {block_hash:#?} != {exp_block_hash:#?}").into(),
        ));
    }

    if block_hash != computed_block_hash {
        println!("Computed block hash mismatch:");
        println!("Block hash output: {block_hash:#?}");
        println!("Computed hash from output block: {computed_block_hash:#?}");
        return Err(HintError::CustomHint(
            format!(
                "Computed hash mismatch: {block_hash:#?} != {computed_block_hash:#?}"
            )
            .into(),
        ));
    }

    if *exp_block_hash != computed_expected_hash {
        println!("Expected block hash is inconsistent with expected block:");
        println!("Expected block hash: {exp_block_hash:#?}");
        println!("Computed hash from expected block: {computed_expected_hash:#?}");
        return Err(HintError::CustomHint(
            format!(
                "Expected hash mismatch: {exp_block_hash:#?} != {computed_expected_hash:#?}"
            )
            .into(),
        ));
    }

    Ok(())
}
