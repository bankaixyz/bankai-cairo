use std::collections::HashMap;

use cairo_vm_base::default_hints::HintImpl;

pub mod input;
pub mod utils;

pub fn get_hints() -> HashMap<String, HintImpl> {
    let mut hints = HashMap::<String, HintImpl>::new();
    hints.insert(
        utils::HINT_CHECK_FORK_VERSION.into(),
        utils::hint_check_fork_version,
    );
    hints.insert(
        utils::HINT_SET_NEXT_POWER_OF_2.into(),
        utils::set_next_power_of_2,
    );
    hints.insert(
        utils::HINT_COMPUTE_EPOCH_FROM_SLOT.into(),
        utils::compute_epoch_from_slot,
    );

    hints
}
