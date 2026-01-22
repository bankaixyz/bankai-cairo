use std::any::Any;
use std::collections::HashMap;

use cairo_vm_base::{
    default_hints::{default_hint_mapping, HintImpl},
    vm::cairo_vm::{
        hint_processor::{
            builtin_hint_processor::builtin_hint_processor_definition::{
                BuiltinHintProcessor, HintProcessorData,
            },
            hint_processor_definition::{HintExtension, HintProcessorLogic},
        },
        types::exec_scope::ExecutionScopes,
        vm::{errors::hint_errors::HintError, runners::cairo_runner::ResourceTracker},
        Felt252,
    },
};

use crate::test_hints::{
    write_block_signer_fixture, write_block_signer_fixtures_len, write_committee_update_fixture,
    write_committee_update_fixtures_len, HINT_WRITE_BLOCK_SIGNER_FIXTURE,
    HINT_WRITE_BLOCK_SIGNER_FIXTURES_LEN, HINT_WRITE_COMMITTEE_UPDATE_FIXTURE,
    HINT_WRITE_COMMITTEE_UPDATE_FIXTURES_LEN,
};
use bankai_hints::hints::get_hints as get_bankai_hints;
use garaga_zero::hints::get_hints as get_garaga_zero_hints;

pub struct TestHintProcessor {
    hints: HashMap<String, HintImpl>,
    builtin_hint_proc: BuiltinHintProcessor,
}

impl Default for TestHintProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TestHintProcessor {
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
        hints.insert(
            HINT_WRITE_BLOCK_SIGNER_FIXTURES_LEN.into(),
            write_block_signer_fixtures_len,
        );
        hints.insert(
            HINT_WRITE_BLOCK_SIGNER_FIXTURE.into(),
            write_block_signer_fixture,
        );
        hints.insert(
            HINT_WRITE_COMMITTEE_UPDATE_FIXTURES_LEN.into(),
            write_committee_update_fixtures_len,
        );
        hints.insert(
            HINT_WRITE_COMMITTEE_UPDATE_FIXTURE.into(),
            write_committee_update_fixture,
        );
        hints
    }
}

impl HintProcessorLogic for TestHintProcessor {
    fn execute_hint(
        &mut self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        self.builtin_hint_proc
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }

    fn execute_hint_extensive(
        &mut self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> Result<HintExtension, HintError> {
        if let Some(hpd) = hint_data.downcast_ref::<HintProcessorData>() {
            let hint_code = hpd.code.as_str();
            if let Some(hint_impl) = self.hints.get(hint_code) {
                return hint_impl(vm, exec_scopes, hpd, constants).map(|_| HintExtension::default());
            }

            return self
                .builtin_hint_proc
                .execute_hint(vm, exec_scopes, hint_data, constants)
                .map(|_| HintExtension::default());
        }

        Err(HintError::WrongHintData)
    }
}

impl ResourceTracker for TestHintProcessor {}
