from starkware.cairo.common.cairo_builtins import (
    BitwiseBuiltin,
    ModBuiltin,
    PoseidonBuiltin,
    HashBuiltin,
)
from starkware.cairo.common.math_cmp import is_le
from starkware.cairo.common.uint256 import Uint256

from cairo.src.light_clients.op_stack.hash import commitment_leaf_hash, hash_op_client_output
from cairo.src.light_clients.op_stack.merkle import update_merkle_leaf
from cairo.src.light_clients.op_stack.types import OpChainInput, OpChainsOutput, OpChainsInput

const OP_STACK_TREE_DEPTH = 5;
const OP_STACK_EMPTY_ROOT_LOW = 0x7736dcf70944067195505a19e433d326;
const OP_STACK_EMPTY_ROOT_HIGH = 0xd686d974150e54f427421b5805b6464c;

func get_empty_op_chains_root() -> (root: Uint256) {
    return (root=Uint256(low=OP_STACK_EMPTY_ROOT_LOW, high=OP_STACK_EMPTY_ROOT_HIGH));
}

func apply_updates{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*}(
    updates: OpChainInput*,
    n_remaining: felt,
    current_root: Uint256,
    max_index: felt,
) -> (new_root: Uint256, max_index: felt) {
    alloc_locals;

    if (n_remaining == 0) {
        return (new_root=current_root, max_index=max_index);
    }

    let update = [updates];
    assert update.prev_merkle_path_len = OP_STACK_TREE_DEPTH;

    let (local old_leaf) = commitment_leaf_hash(update.prev_output);
    let (local new_leaf) = hash_op_client_output(update.output);

    let (local next_root) = update_merkle_leaf(
        path=update.prev_merkle_path,
        path_len=update.prev_merkle_path_len,
        old_leaf=old_leaf,
        new_leaf=new_leaf,
        index=update.client_index,
        expected_root=current_root,
    );

    // Track the highest touched index so n_clients grows for new sparse clients.
    let next_max = is_le(max_index, update.client_index);
    if (next_max == 1) {
        return apply_updates(
            updates=updates + OpChainInput.SIZE,
            n_remaining=n_remaining - 1,
            current_root=next_root,
            max_index=update.client_index,
        );
    }

    return apply_updates(
        updates=updates + OpChainInput.SIZE,
        n_remaining=n_remaining - 1,
        current_root=next_root,
        max_index=max_index,
    );
}


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

    local op_inputs: OpChainsInput;
    %{ write_op_stack_inputs() %}

    assert op_inputs.prev_root.low = prev.root.low;
    assert op_inputs.prev_root.high = prev.root.high;

    if (op_inputs.n_updates == 0) {
        return (output=prev);
    }

    let (local new_root, max_index) = apply_updates(
        updates=op_inputs.updates,
        n_remaining=op_inputs.n_updates,
        current_root=prev.root,
        max_index=0,
    );
    let new_n_clients = max_index + 1;
    let prev_le_new = is_le(prev.n_clients, new_n_clients);
    if (prev_le_new == 1) {
        return (output=OpChainsOutput(root=new_root, n_clients=new_n_clients));
    }

    return (output=OpChainsOutput(root=new_root, n_clients=prev.n_clients));
}