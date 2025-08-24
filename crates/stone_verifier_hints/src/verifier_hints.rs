use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{
    get_relocatable_from_var_name, insert_value_from_var_name,
};
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm::Felt252;
use cairo_vm::{types::exec_scope::ExecutionScopes, vm::errors::hint_errors::HintError};
use num_bigint::BigUint;
use serde_json::from_value;
use std::collections::HashMap;

use super::verifier_utils::{extract_from_ids_and_public_input, gen_arg, get_program_identifies};
use crate::cairo_structs::ToVec;
use crate::types::{ExtractedProofValues, OwnedPublicInput};
use crate::verifier_utils::{extract_proof_values, get_stark_proof_cairo_struct};

use super::types::CairoVerifierInput;
use super::vars::{PROGRAM_INPUT, PROGRAM_OBJECT};

/// Implements
///
/// %{
///     from starkware.cairo.stark_verifier.air.parser import parse_proof
///     ids.proof = segments.gen_arg(parse_proof(
///         identifiers=ids._context.identifiers,
///         proof_json=program_input["proof"]))
/// %}
pub const HINT_LOAD_AND_PARSE_PROOF: &str = r#"from starkware.cairo.stark_verifier.air.parser import parse_proof
ids.proof = segments.gen_arg(parse_proof(
    identifiers=ids._context.identifiers,
    proof_json=program_input["proof"]))"#;

pub fn load_and_parse_proof(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    println!("starting parsing STARK proof");

    let program_input: &String = exec_scopes.get_ref(PROGRAM_INPUT)?;

    let cairo_verifier_input: CairoVerifierInput =
        serde_json::from_str(program_input).map_err(|_| {
            HintError::CustomHint(
                "Failed to deserialize program_input into cairo_verifier_input".into(),
            )
        })?;

    // Retrieve verifier identifiers from execution scopes
    let verifier_identifiers = get_program_identifies(exec_scopes, PROGRAM_OBJECT)?;

    // Extract the "public_input" section from proof
    let public_input_json = cairo_verifier_input
        .proof
        .get("public_input")
        .ok_or_else(|| HintError::CustomHint("Missing 'public_input' in proof JSON.".into()))?;

    // Deserialize "public_input" JSON to OwnedPublicInput struct
    let public_input: OwnedPublicInput = from_value(public_input_json.clone())
        .map_err(|_| HintError::CustomHint("Failed to deserialize 'public_input' JSON.".into()))?;

    // Extract proof values into a struct
    let extracted_proof_vals: ExtractedProofValues =
        extract_proof_values(&cairo_verifier_input.proof)?;

    // Extract identifiers and public input values
    let extracted_ids_and_pub_in_vals = extract_from_ids_and_public_input(
        &public_input,
        &verifier_identifiers,
        &extracted_proof_vals,
    )?;

    // Generate the Cairo-compatible Stark proof structure
    let stark_proof_cairo_struct = get_stark_proof_cairo_struct(
        &extracted_proof_vals,
        &extracted_ids_and_pub_in_vals,
        &public_input,
    )?;
    let proof_relocatable = gen_arg(vm, &stark_proof_cairo_struct.to_vec())?;

    insert_value_from_var_name(
        "proof",
        proof_relocatable,
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    println!("finished parsing STARK proof");

    Ok(())
}

pub const HINT_SET_BIT_FROM_INDEX: &str = r#"ids.bit = ids.current.index & 1"#;

pub fn set_bit_from_index(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    // Retrieve the `index` value from `ids.current`
    let current_addr =
        get_relocatable_from_var_name("current", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;

    // Address is to the base of the following Cairo struct:
    // struct VectorQueryWithDepth {
    //     index: felt,
    //     value: felt,
    //     depth: felt,
    // }
    // We need to get the value of `index` from the struct.
    let index = vm.get_integer(current_addr)?.to_biguint();

    let bit = MaybeRelocatable::Int((index & BigUint::from(1u8)).into());
    insert_value_from_var_name("bit", bit, vm, &hint_data.ids_data, &hint_data.ap_tracking)?;

    Ok(())
}

pub const VERIFIER_DIVIDE_QUERIES_IND_BY_COSET_SIZE_TO_FP_OFFSET: &str =
    "memory[fp + 1] = to_felt_or_relocatable(ids.queries.index // ids.params.coset_size)";

pub fn divide_queries_ind_by_coset_size_to_fp_offset(
    vm: &mut VirtualMachine,
    _exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let queries_base =
        get_relocatable_from_var_name("queries", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    let params_base =
        get_relocatable_from_var_name("params", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;

    // queries_addr is the base of an array of the following Cairo structs:
    // struct FriLayerQuery {
    //     index: felt,
    //     y_value: felt,
    //     x_inv_value: felt,
    // }
    // We need to get the value of `index` from the first struct.
    //
    // params_addr is the base of an array of the following Cairo struct:
    // struct FriLayerComputationParams {
    //     coset_size: felt,
    //     fri_group: felt*,
    //     eval_point: felt,
    // }
    // We need to get the value of `coset_size` from the first struct.

    let first_query_addr = vm.get_relocatable(queries_base).map_err(|_| {
        HintError::CustomHint("Failed to retrieve `first_query_addr` as relocatable.".into())
    })?;

    let first_params_addr = vm.get_relocatable(params_base).map_err(|_| {
        HintError::CustomHint("Failed to retrieve `first_params_addr` as relocatable.".into())
    })?;

    // Perform integer division
    let index = vm
        .get_integer(first_query_addr)
        .map_err(|_| {
            HintError::CustomHint("Failed to retrieve `queries.index` as integer.".into())
        })?
        .to_biguint();

    // Get `coset_size` from `params`
    let coset_size = vm
        .get_integer(first_params_addr)
        .map_err(|_| {
            HintError::CustomHint("Failed to retrieve `params.coset_size` as integer.".into())
        })?
        .to_biguint();
    let result = MaybeRelocatable::Int((index / coset_size).into());

    // Insert result into memory at `fp + 1`
    let dest = (vm.get_fp() + 1)?;
    Ok(vm.insert_value(dest, result)?)
}
