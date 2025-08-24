use std::collections::HashMap;

use cairo_vm_base::cairo_type::{CairoType, CairoWritable};
use cairo_vm_base::types::serde_utils::{deserialize_from_any, deserialize_vec_from_string};
use cairo_vm_base::types::{felt::Felt, uint256::Uint256, uint384::UInt384};
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{get_ptr_from_var_name, get_relocatable_from_var_name};
use cairo_vm_base::vm::cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm_base::vm::cairo_vm::Felt252;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct G1PointCairo {
    #[serde(deserialize_with = "deserialize_from_any")]
    x: UInt384,
    #[serde(deserialize_with = "deserialize_from_any")]
    y: UInt384,
}

#[derive(Debug, Deserialize)]
pub struct G2PointCairo {
    #[serde(deserialize_with = "deserialize_from_any")]
    x0: UInt384,
    #[serde(deserialize_with = "deserialize_from_any")]
    x1: UInt384,
    #[serde(deserialize_with = "deserialize_from_any")]
    y0: UInt384,
    #[serde(deserialize_with = "deserialize_from_any")]
    y1: UInt384,
}