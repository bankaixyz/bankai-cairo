from starkware.cairo.common.cairo_builtins import PoseidonBuiltin
from cairo.src.io import CircuitOutput2
from starkware.cairo.common.builtin_poseidon.poseidon import poseidon_hash_many

func compute_output_hash{range_check_ptr, poseidon_ptr: PoseidonBuiltin*}(
    program_hash: felt, previous_output: CircuitOutput2
) -> (output_hash: felt) {
    alloc_locals;

    tempvar output_elements = cast(
        new (
            1,
            24,
            program_hash,
            previous_output.block_number,
            previous_output.beacon.slot_number,
            previous_output.beacon.header_root.low,
            previous_output.beacon.header_root.high,
            previous_output.beacon.justified_height,
            previous_output.beacon.finalized_height,
            previous_output.beacon.num_signers,
            previous_output.beacon.mmr_root_keccak.low,
            previous_output.beacon.mmr_root_keccak.high,
            previous_output.beacon.mmr_root_poseidon,
            previous_output.beacon.current_committee_hash.low,
            previous_output.beacon.current_committee_hash.high,
            previous_output.beacon.next_committee_hash.low,
            previous_output.beacon.next_committee_hash.high,
            previous_output.execution.block_number,
            previous_output.execution.header_hash.low,
            previous_output.execution.header_hash.high,
            previous_output.execution.justified_height,
            previous_output.execution.finalized_height,
            previous_output.execution.mmr_root_keccak.low,
            previous_output.execution.mmr_root_keccak.high,
            previous_output.execution.mmr_root_poseidon,
        ),
        felt*,
    );

    let (output_hash: felt) = poseidon_hash_many(n=25, elements=output_elements);

    return (output_hash=output_hash);
}
