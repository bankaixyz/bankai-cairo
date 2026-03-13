use cairo_vm_base::{
    cairo_type::{CairoType, CairoWritable},
    types::{felt::Felt, uint256::Uint256},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct OpChainsOutputCairo {
    pub root: Uint256,   // this is the merkle root of all op clients
    pub n_clients: Felt, // number of op clients
}

impl CairoType for OpChainsOutputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.root.to_memory(vm, current_ptr)?;
        current_ptr = self.n_clients.to_memory(vm, current_ptr)?;
        Ok(current_ptr)
    }

    fn from_memory(
        vm: &cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<Self, cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError> {
        Ok(Self {
            root: Uint256::from_memory(vm, address)?,
            n_clients: Felt::from_memory(vm, (address + 2)?)?,
        })
    }

    fn n_fields() -> usize {
        Uint256::n_fields() + Felt::n_fields()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct OpChainsInputCairo {
    pub prev_root: Uint256,              // prev root of all clients
    pub n_updates: Felt,                 // number of updates
    pub updates: Vec<OpChainInputCairo>, // updates some clients
}

impl CairoWritable for OpChainsInputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.prev_root.to_memory(vm, current_ptr)?;
        current_ptr = self.n_updates.to_memory(vm, current_ptr)?;

        let updates_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, updates_segment)?;
        current_ptr = (current_ptr + 1)?;

        let mut updates_ptr = updates_segment;
        for update in &self.updates {
            updates_ptr = update.to_memory(vm, updates_ptr)?;
        }

        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!(
                    "Memory layout mismatch for OpChainsInputCairo: expected pointer at {expected_ptr}, but got {current_ptr}"
                )
                .into(),
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Uint256::n_fields() + Felt::n_fields() + 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct OpChainInputCairo {
    pub client_index: Felt,               // index of a client in the merkle tree
    pub prev_merkle_path: Vec<Uint256>,   // merkle path to the prev client
    pub prev_merkle_path_len: Felt,       // length of the merkle path
    pub prev_output: OpClientOutputCairo, // prev output of the client we need to decommmit
    pub output: OpClientOutputCairo,      // new output of the client we need to commit
}

impl CairoWritable for OpChainInputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.client_index.to_memory(vm, current_ptr)?;

        // Store Uint256* pointer, then write path elements into that segment.
        let prev_merkle_path_segment = vm.add_memory_segment();
        vm.insert_value(current_ptr, prev_merkle_path_segment)?;
        current_ptr = (current_ptr + 1)?;
        current_ptr = self.prev_merkle_path_len.to_memory(vm, current_ptr)?;

        let mut prev_merkle_path_ptr = prev_merkle_path_segment;
        for path_element in &self.prev_merkle_path {
            prev_merkle_path_ptr = path_element.to_memory(vm, prev_merkle_path_ptr)?;
        }

        current_ptr = self.prev_output.to_memory(vm, current_ptr)?;
        current_ptr = self.output.to_memory(vm, current_ptr)?;

        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!(
                    "Memory layout mismatch for OpChainInputCairo: expected pointer at {expected_ptr}, but got {current_ptr}"
                )
                .into(),
            ));
        }

        Ok(current_ptr)
    }

    fn n_fields() -> usize {
        Felt::n_fields() + 2 + OpClientOutputCairo::n_fields() * 2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct OpClientOutputCairo {
    pub chain_id: Felt,
    pub block_number: Felt,
    pub header_hash: Uint256,
    pub l1_submission_block: Felt,
    pub mmr_root_keccak: Uint256,
    pub mmr_root_poseidon: Felt,
}

impl CairoType for OpClientOutputCairo {
    fn to_memory(
        &self,
        vm: &mut cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<
        cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
        cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError,
    > {
        let mut current_ptr = address;
        current_ptr = self.chain_id.to_memory(vm, current_ptr)?;
        current_ptr = self.block_number.to_memory(vm, current_ptr)?;
        current_ptr = self.header_hash.to_memory(vm, current_ptr)?;
        current_ptr = self.l1_submission_block.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_keccak.to_memory(vm, current_ptr)?;
        current_ptr = self.mmr_root_poseidon.to_memory(vm, current_ptr)?;

        let expected_ptr = (address + Self::n_fields())?;
        if current_ptr != expected_ptr {
            return Err(cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError::CustomHint(
                format!(
                    "Memory layout mismatch for OpClientOutputCairo: expected pointer at {expected_ptr}, but got {current_ptr}"
                )
                .into(),
            ));
        }

        Ok(current_ptr)
    }

    fn from_memory(
        vm: &cairo_vm_base::vm::cairo_vm::vm::vm_core::VirtualMachine,
        address: cairo_vm_base::vm::cairo_vm::types::relocatable::Relocatable,
    ) -> Result<Self, cairo_vm_base::vm::cairo_vm::vm::errors::hint_errors::HintError> {
        Ok(Self {
            chain_id: Felt::from_memory(vm, address)?,
            block_number: Felt::from_memory(vm, (address + 1)?)?,
            header_hash: Uint256::from_memory(vm, (address + 2)?)?,
            l1_submission_block: Felt::from_memory(vm, (address + 4)?)?,
            mmr_root_keccak: Uint256::from_memory(vm, (address + 5)?)?,
            mmr_root_poseidon: Felt::from_memory(vm, (address + 7)?)?,
        })
    }

    fn n_fields() -> usize {
        Felt::n_fields() * 4 + Uint256::n_fields() * 2
    }
}
