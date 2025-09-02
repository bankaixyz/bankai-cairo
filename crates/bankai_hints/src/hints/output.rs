use std::collections::HashMap;

use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_address_from_var_name;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_ptr_from_var_name;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::get_relocatable_from_var_name;
use cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm_base::vm::cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm_base::vm::cairo_vm::Felt252;
use cairo_vm_base::cairo_type::CairoWritable;
use cairo_vm_base::cairo_type::CairoType;

use crate::types::CircuitOutputCairo2;

pub const HINT_ASSERT_OUTPUT: &str = r#"assert_output()"#;

pub fn assert_output(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let expected_output = exec_scopes.get_ref::<CircuitOutputCairo2>("output")?;
    let circuit_output_ptr = get_relocatable_from_var_name("output", vm, &hint_data.ids_data, &hint_data.ap_tracking)?;
    let circuit_output = CircuitOutputCairo2::from_memory(vm, circuit_output_ptr)?;

    if circuit_output != *expected_output {
        return Err(HintError::CustomHint(format!("Circuit output do not match expected output: {circuit_output:?} != {expected_output:?}").into()));
    }

    Ok(())
}
