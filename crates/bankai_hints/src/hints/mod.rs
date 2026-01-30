use std::collections::HashMap;

use cairo_vm_base::default_hints::HintImpl;

pub mod os;
pub mod ethereum;

pub fn get_hints() -> HashMap<String, HintImpl> {
    let mut hints = HashMap::<String, HintImpl>::new();
    hints.insert(
        ethereum::HINT_CHECK_FORK_VERSION.into(),
        ethereum::hint_check_fork_version,
    );
    hints.insert(
        ethereum::HINT_WRITE_CONSENSUS_INPUTS.into(),
        ethereum::write_consensus_inputs,
    );
    hints.insert(
        ethereum::HINT_WRITE_COMMITTEE_UPDATE_INPUTS.into(),
        ethereum::write_committee_update_inputs,
    );
    hints.insert(
        os::HINT_WRITE_INIT_DATA.into(),
        os::write_init_data,
    );
    hints.insert(
        os::HINT_PREVIOUS_BLOCK.into(),
        os::write_previous_block,
    );
    hints.insert(
        os::HINT_WRITE_MOCK_RECURSION_INPUTS.into(),
        os::write_mock_recursion_inputs,
    );
    hints.insert(
        os::HINT_VERIFY_BLOCK_RESULT.into(),
        os::verify_block_result,
    );
    hints
}
