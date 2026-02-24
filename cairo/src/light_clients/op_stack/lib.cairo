from starkware.cairo.common.cairo_builtins import (

    BitwiseBuiltin,
    ModBuiltin,
    PoseidonBuiltin,
    HashBuiltin,
)
from cairo.src.debug.print import debug_felt_hex, debug_string, info_felt, info_string
from cairo.src.utils.utils import felt_divmod
from cairo.src.light_clients.op_stack.types import OpChainsOutput, OpChainsInput


func run{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}(prev: OpChainsOutput) -> (
    output: OpChainsOutput
) {
    alloc_locals;

    local expected_output: OpChainsOutput;
    %{ unsafe_write_op_stack_expected_output() %}

    return (output=expected_output);

}