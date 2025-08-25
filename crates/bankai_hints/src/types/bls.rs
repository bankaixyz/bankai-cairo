

use cairo_vm_base::cairo_type::CairoWritable;
use cairo_vm_base::types::{felt::Felt, uint256::Uint256, uint384::UInt384};
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm_base::vm::cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{get_ptr_from_var_name, get_relocatable_from_var_name};
use serde::{Deserialize, Serialize};
use cairo_vm_base::cairo_type::CairoType;

#[derive(Debug, Serialize, Deserialize)]
pub struct G1PointCairo {
    x: UInt384,
    y: UInt384,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct G2PointCairo {
    x0: UInt384,
    x1: UInt384,
    y0: UInt384,
    y1: UInt384,
}

impl CairoWritable for G1PointCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;

        current_ptr = self.x.to_memory(vm, current_ptr)?;
        current_ptr = self.y.to_memory(vm, current_ptr)?;

        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!("Memory layout mismatch for G1PointCairo: expected pointer at {}, but got {}", expected_ptr, current_ptr).into()
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        8
    }
}

impl CairoWritable for G2PointCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;

        current_ptr = self.x0.to_memory(vm, current_ptr)?;
        current_ptr = self.x1.to_memory(vm, current_ptr)?;
        current_ptr = self.y0.to_memory(vm, current_ptr)?;
        current_ptr = self.y1.to_memory(vm, current_ptr)?;

        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!("Memory layout mismatch for G2PointCairo: expected pointer at {}, but got {}", expected_ptr, current_ptr).into()
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        16
    }
}