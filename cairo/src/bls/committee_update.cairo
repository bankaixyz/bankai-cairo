from starkware.cairo.common.cairo_builtins import PoseidonBuiltin, ModBuiltin, BitwiseBuiltin, HashBuiltin
from starkware.cairo.common.bitwise import bitwise_and
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.uint256 import Uint256
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.memset import memset
from definitions import UInt384
from sha import SHA256, HashUtils
from ec_ops import derive_g1_point_from_x
from debug import print_felt_hex, print_uint384, print_string

from cairo.src.utils.domain import Network
from cairo.src.utils.utils import pow2alloc128, felt_divmod
from cairo.src.bls.signer import commit_committee_key
from cairo.src.utils.ssz import MerkleTree

// Compute the leaf hash for the Merkle tree
func compute_leaf_hash{range_check_ptr, pow2_array: felt*, sha256_ptr: felt*}(
    committee_keys_root: felt*, aggregate_committee_key: UInt384
) -> felt* {
    alloc_locals;
    // Step 1: Create leaf hash -> h(sync_committee_root, aggregate_committee_key)
    let (aggregate_committee_key_chunks) = HashUtils.chunk_uint384(aggregate_committee_key);
    // Pad the key to 64 bytes
    memset(dst=aggregate_committee_key_chunks + 12, value=0, n=4);
    let (aggregate_committee_root) = SHA256.hash_bytes(aggregate_committee_key_chunks, 64);

    // Copy the root and compute the final leaf hash
    memcpy(dst=committee_keys_root + 8, src=aggregate_committee_root, len=8);
    let (leaf_hash) = SHA256.hash_bytes(committee_keys_root, 64);
    return leaf_hash;
}

// Compute the hash of the committee point h(x||y)
func compute_committee_hash{
    range_check_ptr,
    sha256_ptr: felt*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*,
}(compressed_g1: UInt384) -> Uint256 {
    alloc_locals;

    // Decompress G1 point and perform sanity checks
    let (flags, x_point) = decompress_g1(compressed_g1);
    assert flags.compression_bit = 1;
    assert flags.infinity_bit = 0;

    // Derive the full G1 point and hash it
    let (point) = derive_g1_point_from_x(curve_id=1, x=x_point, s=flags.sign_bit);
    let committee_hash = commit_committee_key(point=point);

    return committee_hash;
}

// Structure to hold flags for compressed G1 points
struct CompressedG1Flags {
    compression_bit: felt,  // Bit 383
    infinity_bit: felt,  // Bit 382
    sign_bit: felt,  // Bit 381
}

// Decompress a G1 point from its compressed form
func decompress_g1{range_check_ptr}(compressed_g1: UInt384) -> (CompressedG1Flags, UInt384) {
    alloc_locals;

    let limb = compressed_g1.d3;

    // Extract bit 383
    let (compression_bit, remainder) = felt_divmod(limb, 0x800000000000000000000000);

    // Extract bit 382
    let (infinity_bit, remainder) = felt_divmod(remainder, 0x400000000000000000000000);

    // Extract bit 381
    let (sign_bit, uncompressed_x_limb) = felt_divmod(remainder, 0x200000000000000000000000);

    // Construct the x coordinate of the point
    let x_point = UInt384(
        d0=compressed_g1.d0, d1=compressed_g1.d1, d2=compressed_g1.d2, d3=uncompressed_x_limb
    );

    return (CompressedG1Flags(compression_bit, infinity_bit, sign_bit), x_point);
}

// Entrypoint function that can be called by the recursive update circuit
func run_committee_update{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*,
    sha256_ptr: felt*,
}(
    committee_keys_root: felt*,
    path: felt**,
    path_len: felt,
    aggregate_committee_key: UInt384,
    slot: felt
) -> (state_root: Uint256, committee_hash: Uint256) {
    alloc_locals;

    let fork = Network.get_fork_version(Network.SEPOLIA, slot);
    local next_committee_index: felt;
    if (fork == Network.ELECTRA) {
        next_committee_index = 87;
    } else {
        next_committee_index = 55;
    }

    let leaf_hash = compute_leaf_hash(committee_keys_root, aggregate_committee_key);
    
    let state_root = MerkleTree.hash_merkle_path(
        path=path, path_len=path_len, leaf=leaf_hash, index=next_committee_index
    );
    let committee_hash = compute_committee_hash(aggregate_committee_key);

    return (state_root, committee_hash);
}

struct CircuitInput {
    beacon_slot: felt,
    next_sync_committee_branch: Uint256*,
    next_aggregate_sync_committee: UInt384,
    committee_keys_root: Uint256,
}
