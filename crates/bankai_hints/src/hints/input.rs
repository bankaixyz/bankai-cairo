

pub fn write_epoch_update_inputs(
        &self,
        vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        hint_data: &HintProcessorData,
        _constants: &HashMap<String, Felt252>,
    ) -> Result<(), HintError> {
        let epoch_update = &self.recursive_epoch_update.inputs.epoch_update;
        let epoch_update_ptr = get_relocatable_from_var_name(
            "epoch_update",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        write_epoch_update(epoch_update_ptr, epoch_update, vm)?;

        let is_genesis_ptr = get_relocatable_from_var_name(
            "is_genesis",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        let is_genesis = match &self.recursive_epoch_update.inputs.stark_proof {
            Some(_) => 0,
            None => 1,
        };
        vm.insert_value(is_genesis_ptr, Felt252::from(is_genesis))?;

        let is_committee_update_ptr = get_relocatable_from_var_name(
            "is_committee_update",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        let is_committee_update = match &self.recursive_epoch_update.inputs.sync_committee_update {
            Some(_) => 1,
            None => 0,
        };
        vm.insert_value(is_committee_update_ptr, Felt252::from(is_committee_update))?;

        let program_hash_ptr = get_relocatable_from_var_name(
            "program_hash",
            vm,
            &hint_data.ids_data,
            &hint_data.ap_tracking,
        )?;
        let program_hash = Felt252::from_hex_unchecked(
            "0x5b6ff167e72599c14a2e99cac4a6e8db3036db0f0d9acac15d5822ea315287a",
        );
        vm.insert_value(program_hash_ptr, program_hash)?;

        Ok(())
    }