
// use cairo_vm_base::cairo_type::{CairoType, CairoWritable};
use cairo_vm_base::{cairo_type::CairoType, types::felt::Felt};
use serde::{Deserialize, Serialize};


// This is a fully mocked type, so we dont pass any proof data
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct MockRecursionCairo {
    pub program_hash: Felt,
    pub proof_data: Option<serde_json::Value>,
}

impl CairoType for MockRecursionCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.program_hash.to_memory(vm, current_ptr)?;
        Ok(current_ptr)
    }

    fn from_memory(
        vm: &cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<Self, cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError> {
        Ok(Self {
            program_hash: Felt::from_memory(vm, address)?,
            proof_data: None,
        })
    }

    fn n_fields() -> usize {
        Felt::n_fields()
    }   
}