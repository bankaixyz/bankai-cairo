from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.math import split_felt
from starkware.cairo.common.uint256 import Uint256

from src.core.keccak import keccak_uint256s_bigend

from cairo.src.light_clients.op_stack.types import OpClientOutput


func felt_to_uint256{range_check_ptr}(value: felt) -> (res: Uint256) {
    let (high, low) = split_felt(value);
    return (res=Uint256(low=low, high=high));
}

func hash_op_client_output{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*}(
    output: OpClientOutput
) -> (hash: Uint256) {
    alloc_locals;
    let (fields_felt_ptr: felt*) = alloc();
    let fields = cast(fields_felt_ptr, Uint256*);

    let (chain_id) = felt_to_uint256(output.chain_id);
    let (block_number) = felt_to_uint256(output.block_number);
    let (l1_submission_block) = felt_to_uint256(output.l1_submission_block);
    let (mmr_root_poseidon) = felt_to_uint256(output.mmr_root_poseidon);

    assert fields[0] = chain_id;
    assert fields[1] = block_number;
    assert fields[2] = output.header_hash;
    assert fields[3] = l1_submission_block;
    assert fields[4] = output.mmr_root_keccak;
    assert fields[5] = mmr_root_poseidon;

    let (hash) = keccak_uint256s_bigend(n_leafs=6, leafs=fields);
    return (hash=hash);
}

func commitment_leaf_hash{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*}(
    output: OpClientOutput
) -> (hash: Uint256) {
    let (hash) = hash_op_client_output(output);
    return (hash=hash);
}
