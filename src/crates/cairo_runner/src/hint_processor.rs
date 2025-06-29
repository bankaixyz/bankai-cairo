use std::{any::Any, collections::HashMap};

use crate::recursive_epoch::{RecursiveEpochUpdateCairo, HINT_WRITE_EXPECTED_PROOF_OUTPUT};
use cairo_vm::{
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
use garaga_zero::*;

use super::hints;

// use crate::{
//     committee_update::{
//         CommitteeUpdateCircuit, HINT_ASSERT_COMMITTEE_UPDATE_RESULT,
//         HINT_WRITE_COMMITTEE_UPDATE_INPUTS,
//     },
//     epoch_batch::{
//         self, EpochUpdateBatchCircuit, HINT_ASSERT_BATCHED_EPOCH_OUTPUTS,
//         HINT_ASSERT_EPOCH_BATCH_OUTPUTS, HINT_WRITE_EPOCH_UPDATE_BATCH_INPUTS,
//     },
//     epoch_update::{
//         self, EpochUpdateCircuit, HINT_ASSERT_EPOCH_UPDATE_RESULT, HINT_WRITE_EPOCH_UPDATE_INPUTS,
//     },
// };
use super::recursive_epoch::{
    HINT_WRITE_COMMITTEE_UPDATE_INPUTS, HINT_WRITE_EPOCH_UPDATE_INPUTS,
    HINT_WRITE_STARK_PROOF_INPUTS,
};

pub type HintImpl = fn(
    &mut VirtualMachine,
    &mut ExecutionScopes,
    &HintProcessorData,
    &HashMap<String, Felt252>,
) -> Result<(), HintError>;

pub struct CustomHintProcessor {
    hints: HashMap<String, HintImpl>,
    // Add the builtin hint processor
    builtin_hint_proc: BuiltinHintProcessor,
    pub recursive_epoch_update: RecursiveEpochUpdateCairo,
}

impl CustomHintProcessor {
    pub fn new(recursive_epoch_update: RecursiveEpochUpdateCairo) -> Self {
        Self {
            hints: Self::hints(),
            builtin_hint_proc: BuiltinHintProcessor::new_empty(),
            recursive_epoch_update,
        }
    }

    fn hints() -> HashMap<String, HintImpl> {
        let mut hints = HashMap::<String, HintImpl>::new();
        hints.insert(
            circuits::HINT_RUN_MODULO_CIRCUIT.into(),
            circuits::run_modulo_circuit,
        );
        hints.insert(
            circuits::HINT_RUN_EXTENSION_FIELD_MODULO_CIRCUIT.into(),
            circuits::run_extension_field_modulo_circuit,
        );
        hints.insert(
            utils::HINT_RETRIEVE_OUTPUT.into(),
            utils::hint_retrieve_output,
        );
        hints.insert(
            basic_field_ops::HINT_UINT384_IS_LE.into(),
            basic_field_ops::hint_uint384_is_le,
        );
        hints.insert(
            basic_field_ops::HINT_ADD_MOD_CIRCUIT.into(),
            basic_field_ops::hint_add_mod_circuit,
        );
        hints.insert(
            basic_field_ops::HINT_NOT_ZERO_MOD_P.into(),
            basic_field_ops::hint_not_zero_mod_p,
        );
        hints.insert(
            basic_field_ops::HINT_SUM_INV_MOD_P.into(),
            basic_field_ops::hint_sum_inv_mod_p,
        );
        hints.insert(
            basic_field_ops::HINT_ADD_MUL_MOD_CIRCUIT.into(),
            basic_field_ops::hint_add_mul_mod_circuit,
        );
        hints.insert(
            basic_field_ops::HINT_IS_ZERO_MOD_P.into(),
            basic_field_ops::hint_is_zero_mod_p,
        );
        hints.insert(
            basic_field_ops::HINT_ASSERT_NEQ_MOD_P.into(),
            basic_field_ops::hint_assert_neq_mod_p,
        );
        hints.insert(
            basic_field_ops::HINT_IS_EQ_MOD_P.into(),
            basic_field_ops::hint_is_eq_mod_p,
        );
        hints.insert(
            basic_field_ops::HINT_IS_OPPOSITE_MOD_P.into(),
            basic_field_ops::hint_is_opposite_mod_p,
        );
        hints.insert(
            basic_field_ops::HINT_ASSERT_NOT_ZERO_MOD_P.into(),
            basic_field_ops::hint_assert_not_zero_mod_p,
        );
        hints.insert(
            utils::HINT_WRITE_FELTS_TO_VALUE_SEGMENT_1.into(),
            utils::hint_write_felts_to_value_segment_1,
        );
        hints.insert(
            utils::HINT_WRITE_FELTS_TO_VALUE_SEGMENT_2.into(),
            utils::hint_write_felts_to_value_segment_2,
        );
        hints.insert(
            utils::HINT_WRITE_FELTS_TO_VALUE_SEGMENT_3.into(),
            utils::hint_write_felts_to_value_segment_3,
        );
        hints.insert(
            utils::HINT_HASH_FULL_TRANSCRIPT_AND_GET_Z_4_LIMBS_1.into(),
            utils::hint_hash_full_transcript_and_get_z_4_limbs_1,
        );
        hints.insert(
            utils::HINT_HASH_FULL_TRANSCRIPT_AND_GET_Z_4_LIMBS_2.into(),
            utils::hint_hash_full_transcript_and_get_z_4_limbs_2,
        );
        hints.insert(
            hash_to_curve::HINT_MAP_TO_CURVE_G2.into(),
            hash_to_curve::hint_map_to_curve_g2,
        );
        hints.insert(
            sha256::HINT_SHA256_FINALIZE.into(),
            sha256::hint_sha256_finalize,
        );
        hints.insert(debug::PRINT_FELT_HEX.into(), debug::print_felt_hex);
        hints.insert(debug::PRINT_FELT.into(), debug::print_felt);
        hints.insert(debug::PRINT_STRING.into(), debug::print_string);
        hints.insert(debug::PRINT_UINT384.into(), debug::print_uint384);

        hints.insert(
            hints::HINT_CHECK_FORK_VERSION.into(),
            hints::hint_check_fork_version,
        );
        hints.insert(
            hints::HINT_SET_NEXT_POWER_OF_2.into(),
            hints::set_next_power_of_2,
        );
        hints.insert(
            hints::HINT_COMPUTE_EPOCH_FROM_SLOT.into(),
            hints::compute_epoch_from_slot,
        );
        hints.insert(
            stone_verifier_hints::verifier_hints::HINT_LOAD_AND_PARSE_PROOF.into(),
            stone_verifier_hints::verifier_hints::load_and_parse_proof,
        );
        hints.insert(
            stone_verifier_hints::verifier_hints::HINT_SET_BIT_FROM_INDEX.into(),
            stone_verifier_hints::verifier_hints::set_bit_from_index,
        );
        hints.insert(
            stone_verifier_hints::verifier_hints::VERIFIER_DIVIDE_QUERIES_IND_BY_COSET_SIZE_TO_FP_OFFSET.into(),
            stone_verifier_hints::verifier_hints::divide_queries_ind_by_coset_size_to_fp_offset,
        );
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
        // Delegate to the builtin hint processor
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
                HINT_WRITE_EPOCH_UPDATE_INPUTS => {
                    self.write_epoch_update_inputs(vm, exec_scopes, hpd, constants)
                }
                HINT_WRITE_STARK_PROOF_INPUTS => {
                    self.write_stark_proof_inputs(vm, exec_scopes, hpd, constants)
                }
                HINT_WRITE_COMMITTEE_UPDATE_INPUTS => {
                    self.write_committee_update_inputs(vm, exec_scopes, hpd, constants)
                }
                HINT_WRITE_EXPECTED_PROOF_OUTPUT => {
                    self.write_expected_proof_output(vm, exec_scopes, hpd, constants)
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
