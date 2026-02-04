from starkware.cairo.common.cairo_builtins import (
    PoseidonBuiltin,
    ModBuiltin,
    BitwiseBuiltin,
    HashBuiltin,
)
from starkware.cairo.common.bitwise import bitwise_and
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.uint256 import Uint256
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.memset import memset
from starkware.cairo.common.math_cmp import is_le

from definitions import UInt384
from sha import SHA256, HashUtils
from ec_ops import derive_g1_point_from_x
from cairo.src.debug.print import info_string, debug_string, debug_felt, debug_felt_hex

from cairo.src.light_clients.ethereum.utils.domain import Fork
from cairo.src.utils.utils import pow2alloc128, felt_divmod
from cairo.src.light_clients.ethereum.bls.signer import validator_commitment
from cairo.src.light_clients.ethereum.utils.ssz import MerkleTree, MerkleUtils
from cairo.src.light_clients.ethereum.utils.merkle import PoseidonMerkleTree
from cairo.src.light_clients.ethereum.types import SyncCommitteeUpdateInputs
from cairo.src.light_clients.ethereum.config.types import Networks, Hardforks

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
}(committee_input: SyncCommitteeUpdateInputs) -> (
    state_root: Uint256, validator_root: felt
) {
    alloc_locals;
    debug_string('committee update');

    let (_, fork) = Fork.get_id(Networks.SEPOLIA, committee_input.slot);
    local next_committee_index: felt;
    let old_index = is_le(fork, Hardforks.DENEB);
    if (old_index == 1) {
        next_committee_index = 55;
    } else {
        next_committee_index = 87;
    }
    debug_string('next committee index');
    debug_felt(next_committee_index);

    let committee_root = compute_committee_root(committee_input.validator_pubs);
    let committee_root_chunks = MerkleUtils.chunk_uint256(committee_root);
    let leaf_hash = compute_leaf_hash(committee_root_chunks, committee_input.aggregate_committee_key);

    let state_root = MerkleTree.hash_merkle_path(
        path=committee_input.path,
        path_len=committee_input.path_len,
        leaf=leaf_hash,
        index=next_committee_index,
    );
    let validator_root = build_validator_tree(committee_input.validator_pubs);
    debug_string('committee update ok');

    return (state_root, validator_root);
}

// In this function, we pass the compressed validator pubs and hash them using sha256, according to the SSZ spec
func compute_committee_root{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, pow2_array: felt*, sha256_ptr: felt*}(
    committee_keys: UInt384*
) -> Uint256 {
    alloc_locals;

    let (ssz_leafs: Uint256*) = alloc();
    compute_committee_root_inner(committee_keys, 0, ssz_leafs);

    let root = MerkleTree.compute_root(leafs=ssz_leafs, leafs_len=512);
    return root;
}

func compute_committee_root_inner{range_check_ptr, pow2_array: felt*, sha256_ptr: felt*}(
    committee_keys: UInt384*, counter: felt, result: Uint256*
) {
    alloc_locals;
    if (counter == 512) {
        return ();
    }

    let (aggregate_committee_key_chunks) = HashUtils.chunk_uint384(committee_keys[counter]);
    // Pad the key to 64 bytes
    memset(dst=aggregate_committee_key_chunks + 12, value=0, n=4);
    let (aggregate_committee_root) = SHA256.hash_bytes(aggregate_committee_key_chunks, 64);

    let val = MerkleUtils.chunks_to_uint256(aggregate_committee_root);

    assert result[counter] = val;

    return compute_committee_root_inner(committee_keys, counter + 1, result);
}

func build_validator_tree{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    range_check96_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*
}(pubs: UInt384*) -> felt {
    alloc_locals;

    let (commitments: felt*) = alloc();
    compute_validator_pub_commitments(pubs, 0, commitments);

    let val_root = PoseidonMerkleTree.compute_root(leafs=commitments, leafs_len=512);
    return val_root;
}

// Decompress the validator points, hash using poseidon, and write to array
func compute_validator_pub_commitments{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    range_check96_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*
}(pubs: UInt384*, counter: felt, result: felt*) {
    alloc_locals;

    if (counter == 512) {
        return ();
    }

    // Decompress G1 point and perform sanity checks
    let (flags, x_point) = decompress_g1(pubs[counter]);
    assert flags.compression_bit = 1;
    assert flags.infinity_bit = 0;

    let (point) = derive_g1_point_from_x(curve_id=1, x=x_point, s=flags.sign_bit);
    let (commitment) = validator_commitment(point);

    assert result[counter] = commitment;

    return compute_validator_pub_commitments(pubs, counter + 1, result);
}

struct CircuitInput {
    beacon_slot: felt,
    next_sync_committee_branch: Uint256*,
    next_aggregate_sync_committee: UInt384,
    committee_keys_root: Uint256,
}
