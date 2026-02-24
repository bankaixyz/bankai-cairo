use bankai_hints::hints::ethereum::{
    HINT_UNSAFE_WRITE_OP_STACK_EXPECTED_OUTPUT, HINT_WRITE_COMMITTEE_UPDATE_INPUTS, HINT_WRITE_CONSENSUS_INPUTS, unsafe_write_op_stack_expected_output, write_committee_update_inputs, write_consensus_inputs
};
use bankai_hints::hints::get_hints as get_bankai_hints;
use bankai_hints::hints::os::{
    verify_block_result, write_init_data, write_mock_recursion_inputs, write_previous_block,
    HINT_PREVIOUS_BLOCK, HINT_VERIFY_BLOCK_RESULT, HINT_WRITE_INIT_DATA,
    HINT_WRITE_MOCK_RECURSION_INPUTS,
};
use cairo_vm_base::default_hints::{default_hint_mapping, HintImpl};
use cairo_vm_base::vm::cairo_vm::{
    hint_processor::{
        builtin_hint_processor::builtin_hint_processor_definition::{
            BuiltinHintProcessor, HintProcessorData,
        },
        hint_processor_definition::{HintExtension, HintProcessorLogic},
    },
    types::exec_scope::ExecutionScopes,
    vm::{
        errors::hint_errors::HintError, runners::cairo_runner::ResourceTracker,
        vm_core::VirtualMachine,
    },
    Felt252,
};
use garaga_zero::hints::get_hints as get_garaga_zero_hints;
use mmr_header_accumulator_hints::hints::get_hints as get_mmr_header_accumulator_hints;
use mmr_header_accumulator_hints::hints::input::{
    write_bankai_input, write_beacon_input, write_execution_input, HINT_WRITE_BANKAI_INPUT,
    HINT_WRITE_BEACON_INPUT, HINT_WRITE_EXECUTION_INPUT,
};
use std::any::Any;
use std::collections::HashMap;

pub struct CustomHintProcessor {
    hints: HashMap<String, HintImpl>,
    builtin_hint_proc: BuiltinHintProcessor,
}

impl Default for CustomHintProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomHintProcessor {
    pub fn new() -> Self {
        Self {
            hints: Self::hints(),
            builtin_hint_proc: BuiltinHintProcessor::new_empty(),
        }
    }

    fn hints() -> HashMap<String, HintImpl> {
        let mut hints = default_hint_mapping();
        hints.extend(get_garaga_zero_hints());
        hints.extend(get_bankai_hints());
        hints.extend(get_mmr_header_accumulator_hints());
        hints
    }
}

impl HintProcessorLogic for CustomHintProcessor {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        self.builtin_hint_proc
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    fn execute_hint_extensive(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<HintExtension, HintError> {
        if let Some(hpd) = hint_data.downcast_ref::<HintProcessorData>() {
            let hint_code = hpd.code.as_str();

            let res = match hint_code {
                HINT_WRITE_CONSENSUS_INPUTS => {
                    write_consensus_inputs(vm, exec_scopes, hpd, constants)
                }
                HINT_WRITE_COMMITTEE_UPDATE_INPUTS => {
                    write_committee_update_inputs(vm, exec_scopes, hpd, constants)
                }
                HINT_WRITE_INIT_DATA => write_init_data(vm, exec_scopes, hpd, constants),
                HINT_PREVIOUS_BLOCK => write_previous_block(vm, exec_scopes, hpd, constants),
                HINT_WRITE_MOCK_RECURSION_INPUTS => {
                    write_mock_recursion_inputs(vm, exec_scopes, hpd, constants)
                }
                HINT_VERIFY_BLOCK_RESULT => verify_block_result(vm, exec_scopes, hpd, constants),
                HINT_WRITE_BEACON_INPUT => write_beacon_input(vm, exec_scopes, hpd, constants),
                HINT_WRITE_EXECUTION_INPUT => {
                    write_execution_input(vm, exec_scopes, hpd, constants)
                }
                HINT_WRITE_BANKAI_INPUT => write_bankai_input(vm, exec_scopes, hpd, constants),
                HINT_UNSAFE_WRITE_OP_STACK_EXPECTED_OUTPUT => {
                    unsafe_write_op_stack_expected_output(vm, exec_scopes, hpd, constants)
                }
                _ => Err(HintError::UnknownHint(
                    hint_code.to_string().into_boxed_str(),
                )),
            };

            if !matches!(res, Err(HintError::UnknownHint(_))) {
                return res.map(|_| HintExtension::default());
            }

            // First try our custom hints
            if let Some(hint_impl) = self.hints.get(hint_code) {
                return hint_impl(vm, exec_scopes, hpd, constants)
                    .map(|_| HintExtension::default());
            }

            // If not found, try the builtin hint processor
            return self
                .builtin_hint_proc
                .execute_hint(vm, exec_scopes, hint_data, constants)
                .map(|_| HintExtension::default());
        }

        Err(HintError::WrongHintData)
    }
}

impl ResourceTracker for CustomHintProcessor {}
