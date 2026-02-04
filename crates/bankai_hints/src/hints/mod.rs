use std::collections::HashMap;

use cairo_vm_base::default_hints::HintImpl;

pub mod ethereum;
pub mod os;

pub fn get_hints() -> HashMap<String, HintImpl> {
    let mut hints = HashMap::<String, HintImpl>::new();
    hints.insert(
        ethereum::HINT_CHECK_FORK_VERSION.into(),
        ethereum::hint_check_fork_version,
    );
    hints
}
