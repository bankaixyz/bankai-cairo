from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.uint256 import Uint256

from src.core.keccak import keccak_uint256_pair_bigend

from cairo.src.utils.utils import felt_divmod


func hash_merkle_path{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*}(
    path: Uint256*, path_len: felt, leaf: Uint256, index: felt
) -> (root: Uint256) {
    alloc_locals;

    if (path_len == 0) {
        return (root=leaf);
    }

    let (next_index, remainder) = felt_divmod(index, 2);
    if (remainder == 0) {
        let (next_leaf) = keccak_uint256_pair_bigend(leaf, [path]);
        return hash_merkle_path(
            path=path + Uint256.SIZE, path_len=path_len - 1, leaf=next_leaf, index=next_index
        );
    }

    let (next_leaf) = keccak_uint256_pair_bigend([path], leaf);
    return hash_merkle_path(
        path=path + Uint256.SIZE, path_len=path_len - 1, leaf=next_leaf, index=next_index
    );
}

func verify_merkle_path{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*}(
    path: Uint256*, path_len: felt, leaf: Uint256, index: felt, expected_root: Uint256
) {
    let (root) = hash_merkle_path(path=path, path_len=path_len, leaf=leaf, index=index);
    assert root.low = expected_root.low;
    assert root.high = expected_root.high;
    return ();
}

func update_merkle_leaf{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*}(
    path: Uint256*,
    path_len: felt,
    old_leaf: Uint256,
    new_leaf: Uint256,
    index: felt,
    expected_root: Uint256,
) -> (new_root: Uint256) {
    verify_merkle_path(
        path=path, path_len=path_len, leaf=old_leaf, index=index, expected_root=expected_root
    );
    let (new_root) = hash_merkle_path(path=path, path_len=path_len, leaf=new_leaf, index=index);
    return (new_root=new_root);
}
