use std::collections::HashMap;

use bankai_hints::types::{bls::G1PointCairo, SyncCommitteeSignerInputCairo};
use cairo_vm_base::{
    cairo_type::CairoWritable,
    types::{felt::Felt, uint384::UInt384, FromAnyStr},
    vm::cairo_vm::{
        hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData,
        hint_processor::builtin_hint_processor::hint_utils::{
            get_integer_from_var_name, get_relocatable_from_var_name,
        },
        types::exec_scope::ExecutionScopes,
        vm::{errors::hint_errors::HintError, vm_core::VirtualMachine},
        Felt252,
    },
};
use serde::Deserialize;

pub const HINT_WRITE_BLOCK_SIGNER_FIXTURES_LEN: &str = r#"write_block_signer_fixtures_len()"#;
pub const HINT_WRITE_BLOCK_SIGNER_FIXTURE: &str = r#"write_block_signer_fixture()"#;

#[derive(Debug, Clone, Deserialize)]
pub struct BlockSignerFixture {
    pub validator_root: String,
    pub signers: Vec<G1PointHex>,
    pub indexes: Vec<String>,
    pub proofs: Vec<Vec<String>>,
    pub proofs_len: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct G1PointHex {
    pub x: String,
    pub y: String,
}

pub fn write_block_signer_fixtures_len(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let fixtures = exec_scopes.get_ref::<Vec<BlockSignerFixture>>("block_signer_fixtures")?;
    let n_fixtures_ptr = get_relocatable_from_var_name(
        "n_fixtures",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    vm.insert_value(n_fixtures_ptr, Felt252::from(fixtures.len() as u64))?;
    Ok(())
}

pub fn write_block_signer_fixture(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    hint_data: &HintProcessorData,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let fixtures = exec_scopes.get_ref::<Vec<BlockSignerFixture>>("block_signer_fixtures")?;
    let index = get_integer_from_var_name("index", vm, &hint_data.ids_data, &hint_data.ap_tracking)?
        .to_biguint();
    let index: usize = index
        .try_into()
        .map_err(|_| HintError::CustomHint("Invalid fixture index".into()))?;

    let fixture = fixtures
        .get(index)
        .ok_or_else(|| HintError::CustomHint("Fixture index out of bounds".into()))?;

    let signer_input = convert_block_signer_fixture(fixture)?;
    let signer_data_ptr = get_relocatable_from_var_name(
        "signer_data",
        vm,
        &hint_data.ids_data,
        &hint_data.ap_tracking,
    )?;
    signer_input.to_memory(vm, signer_data_ptr)?;
    Ok(())
}

fn convert_block_signer_fixture(
    fixture: &BlockSignerFixture,
) -> Result<SyncCommitteeSignerInputCairo, HintError> {
    let validator_root =
        Felt::from_any_str(&fixture.validator_root).map_err(|e| HintError::CustomHint(e.into()))?;
    let proofs_len =
        Felt::from_any_str(&fixture.proofs_len).map_err(|e| HintError::CustomHint(e.into()))?;

    let mut signers = Vec::with_capacity(fixture.signers.len());
    for signer in &fixture.signers {
        let x = UInt384::from_any_str(&signer.x).map_err(|e| HintError::CustomHint(e.into()))?;
        let y = UInt384::from_any_str(&signer.y).map_err(|e| HintError::CustomHint(e.into()))?;
        signers.push(G1PointCairo::new(x, y));
    }

    let mut indexes = Vec::with_capacity(fixture.indexes.len());
    for index in &fixture.indexes {
        let felt = Felt::from_any_str(index).map_err(|e| HintError::CustomHint(e.into()))?;
        indexes.push(felt);
    }

    let mut proofs = Vec::with_capacity(fixture.proofs.len());
    for proof in &fixture.proofs {
        let mut path = Vec::with_capacity(proof.len());
        for node in proof {
            let felt = Felt::from_any_str(node).map_err(|e| HintError::CustomHint(e.into()))?;
            path.push(felt);
        }
        proofs.push(path);
    }

    Ok(SyncCommitteeSignerInputCairo {
        validator_root,
        signers,
        indexes,
        proofs,
        proofs_len,
    })
}
