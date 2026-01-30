from starkware.cairo.common.cairo_builtins import PoseidonBuiltin, BitwiseBuiltin, HashBuiltin

func compute_output_hash{range_check_ptr, poseidon_ptr: PoseidonBuiltin*}(
    program_hash: felt, output_ptr: felt*, output_len: felt
) -> (output_hash: felt) {
    // TODO: hash output segment (include program_hash prefix)
    return (output_hash=0);
}

func verify_previous_proof{
    range_check_ptr,
    pedersen_ptr: HashBuiltin*,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
}() -> (program_hash: felt, output_hash: felt) {
    // TODO: verify proof and extract program_hash/output_hash
    // TODO(hardfork): allow program_hash upgrade based on governance proof or epoch
    return (program_hash=0, output_hash=0);
}
