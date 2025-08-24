

use cairo_vm_base::types::{felt::Felt, uint256::Uint256, uint384::UInt384};
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{get_ptr_from_var_name, get_relocatable_from_var_name};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct G1PointCairo {
    x: UInt384,
    y: UInt384,
}

#[derive(Debug, Deserialize)]
pub struct G2PointCairo {
    x0: UInt384,
    x1: UInt384,
    y0: UInt384,
    y1: UInt384,
}