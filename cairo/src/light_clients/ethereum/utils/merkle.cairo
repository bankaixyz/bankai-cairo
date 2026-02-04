from starkware.cairo.common.builtin_poseidon.poseidon import poseidon_hash
from starkware.cairo.common.cairo_builtins import PoseidonBuiltin
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.memcpy import memcpy
from cairo.src.utils.utils import felt_divmod

namespace PoseidonMerkleTree {
    func compute_root{range_check_ptr, poseidon_ptr: PoseidonBuiltin*, pow2_array: felt*}(
        leafs: felt*, leafs_len: felt
    ) -> felt {
        alloc_locals;

        // It needs to be ensured that the leafs_len is a power of 2.

        let (tree: felt*) = alloc();
        let tree_len = 2 * leafs_len - 1;  // total nodes in the tree

        // copy the leafs to the end of the tree array
        memcpy(dst=tree + (tree_len - leafs_len), src=leafs, len=leafs_len);

        // Calculate number of internal nodes to process
        let internal_nodes = leafs_len - 1;

        // Set up initial pointers:
        // tree_ptr starts at the last pair of leaves
        let tree_ptr = tree + tree_len;
        // out_ptr starts where first set of hashes should be written
        let out_ptr = tree + internal_nodes;

        compute_merkle_root_inner_optimized(
            tree_ptr=tree_ptr, out_ptr=out_ptr, steps=internal_nodes
        );

        // The root will be at the first position of the array
        return [tree];
    }

    // tree_ptr: pointing to where we read children for hashing
    // out_ptr: pointing to where we place the newly computed hash
    // steps: number of internal nodes to compute
    func compute_merkle_root_inner_optimized{range_check_ptr, poseidon_ptr: PoseidonBuiltin*}(
        tree_ptr: felt*, out_ptr: felt*, steps: felt
    ) {
        alloc_locals;

        // Base case: no more internal nodes to compute
        if (steps == 0) {
            return ();
        }

        // Move read pointer back by 2 to get the pair to hash
        tempvar new_tree_ptr = tree_ptr - 2;

        // Hash the pair of nodes
        let (node) = poseidon_hash([new_tree_ptr], [new_tree_ptr + 1]);

        // Store result and move write pointer back by 1
        tempvar new_out_ptr = out_ptr - 1;
        assert [new_out_ptr] = node;

        // Continue with remaining nodes
        return compute_merkle_root_inner_optimized(
            tree_ptr=new_tree_ptr, out_ptr=new_out_ptr, steps=steps - 1
        );
    }

    // Computes the Merkle root from a leaf, its sibling path and index
    func hash_merkle_path_poseidon{
        range_check_ptr, poseidon_ptr: PoseidonBuiltin*, pow2_array: felt*
    }(path: felt*, path_len: felt, leaf: felt, index: felt) -> felt {
        alloc_locals;

        if (path_len == 0) {
            return leaf;
        }

        let (new_index, r) = felt_divmod(index, 2);
        if (r == 0) {
            let (next_leaf) = poseidon_hash(leaf, [path]);
            return hash_merkle_path_poseidon(
                path=path + 1, path_len=path_len - 1, leaf=next_leaf, index=new_index
            );
        } else {
            let (next_leaf) = poseidon_hash([path], leaf);
            return hash_merkle_path_poseidon(
                path=path + 1, path_len=path_len - 1, leaf=next_leaf, index=new_index
            );
        }
    }

    // Verifies a Merkle proof against an expected root
    func verify_merkle_path_poseidon{
        range_check_ptr, poseidon_ptr: PoseidonBuiltin*, pow2_array: felt*
    }(path: felt*, path_len: felt, leaf: felt, index: felt, expected_root: felt) {
        alloc_locals;
        let root = hash_merkle_path_poseidon(
            path=path, path_len=path_len, leaf=leaf, index=index
        );
        assert root = expected_root;
        return ();
    }
}
