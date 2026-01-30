pub mod recursion;

use cairo_vm_base::cairo_type::CairoType;
use cairo_vm_base::types::{
    felt::Felt
};
use serde::{Deserialize, Serialize};

use crate::types::ethereum::{EthereumClientOutputCairo, EthereumInputsCairo};
use crate::types::os::recursion::MockRecursionCairo;


#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BankaiBlockCairo {
    pub version: Felt,
    pub program_hash: Felt,
    pub block_number: Felt,
    pub ethereum: EthereumClientOutputCairo,
}

impl CairoType for BankaiBlockCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.version.to_memory(vm, current_ptr)?;
        current_ptr = self.program_hash.to_memory(vm, current_ptr)?;
        current_ptr = self.block_number.to_memory(vm, current_ptr)?;
        current_ptr = self.ethereum.to_memory(vm, current_ptr)?;
        Ok(current_ptr)
    }

    fn from_memory(
        vm: &cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<Self, cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError> {
        Ok(Self {
            version: Felt::from_memory(vm, address)?,
            program_hash: Felt::from_memory(vm, (address + 1)?)?,
            block_number: Felt::from_memory(vm, (address + 2)?)?,
            ethereum: EthereumClientOutputCairo::from_memory(vm, (address + 3)?)?,
        })
    }

    fn n_fields() -> usize {
        Felt::n_fields() * 4 + EthereumClientOutputCairo::n_fields()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BankaiBlockInputsCairo {
    pub recursion: MockRecursionCairo,
    pub prev: BankaiBlockCairo,
    pub ethereum: EthereumInputsCairo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BankaiBlockBundleCairo {
    // everything we need to proceed to the next block
    pub inputs: BankaiBlockInputsCairo,
    // the expected output of the next block
    pub output: BankaiBlockCairo,
}

