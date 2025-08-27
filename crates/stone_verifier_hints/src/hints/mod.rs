use std::collections::HashMap;

use cairo_vm_base::default_hints::HintImpl;

pub mod verifier_hints;

pub fn get_hints() -> HashMap<String, HintImpl> {
    let mut hints = HashMap::<String, HintImpl>::new();
    hints.insert(
        verifier_hints::HINT_LOAD_AND_PARSE_PROOF.into(),
        verifier_hints::load_and_parse_proof,
    );
    hints.insert(
        verifier_hints::HINT_SET_BIT_FROM_INDEX.into(),
        verifier_hints::set_bit_from_index,
    );
    hints.insert(
        verifier_hints::VERIFIER_DIVIDE_QUERIES_IND_BY_COSET_SIZE_TO_FP_OFFSET.into(),
        verifier_hints::divide_queries_ind_by_coset_size_to_fp_offset,
    );

    hints
}
