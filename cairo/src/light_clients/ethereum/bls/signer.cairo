from starkware.cairo.common.cairo_builtins import ModBuiltin, PoseidonBuiltin
from starkware.cairo.common.uint256 import Uint256
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.builtin_poseidon.poseidon import poseidon_hash_many

from definitions import G1Point
from ec_ops import add_ec_points
from sha import HashUtils, SHA256
from cairo.src.light_clients.ethereum.utils.merkle import PoseidonMerkleTree
from cairo.src.light_clients.ethereum.types import SyncCommitteeSignerInput

func generate_block_signer_pub{
    range_check_ptr,
    pow2_array: felt*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    sha256_ptr: felt*,
}(signer_data: SyncCommitteeSignerInput) -> (agg_pub: G1Point) {
    alloc_locals;

    assert_signers_inclusion(signer_data, 0);
    let (agg_pub) = aggregate_signer_pubs(signer_data, 0);
    return (agg_pub=agg_pub);
}

func assert_signers_inclusion{
    range_check_ptr,
    pow2_array: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    sha256_ptr: felt*,
}(signer_data: SyncCommitteeSignerInput, counter: felt) {
    alloc_locals;

    if (counter == signer_data.n_signers) {
        return ();
    }

    let (commitment) = validator_commitment(signer_data.signers[counter]);
    PoseidonMerkleTree.verify_merkle_path_poseidon(
        path=signer_data.proofs[counter],
        path_len=signer_data.proofs_len,
        leaf=commitment,
        index=signer_data.indexes[counter],
        expected_root=signer_data.validator_root
    );

    return assert_signers_inclusion(signer_data, counter + 1);
}

func aggregate_signer_pubs{
    range_check_ptr, range_check96_ptr: felt*, add_mod_ptr: ModBuiltin*, mul_mod_ptr: ModBuiltin*
}(signer_data: SyncCommitteeSignerInput, counter: felt) -> (res: G1Point) {
    if (counter == signer_data.n_signers - 1) {
        return (signer_data.signers[counter],);
    }

    let (tail_res) = aggregate_signer_pubs(signer_data, counter + 1);
    return add_ec_points(1, tail_res, signer_data.signers[counter]);
}

// This function generates the hash of an aggregate committee key.
// This hash is stored in the cairo1 state, and is used to check if the correct committee was used
func commit_committee_key{range_check_ptr, sha256_ptr: felt*, pow2_array: felt*}(
    point: G1Point
) -> Uint256 {
    alloc_locals;

    let (x_chunks) = HashUtils.chunk_uint384(point.x);
    let (y_chunks) = HashUtils.chunk_uint384(point.y);

    // Concatenate x and y chunks and compute the hash
    memcpy(dst=x_chunks + 12, src=y_chunks, len=12);
    let (committee_point_hash_chunks) = SHA256.hash_bytes(x_chunks, 96);
    let committee_point_hash = HashUtils.chunks_to_uint256(committee_point_hash_chunks);

    return committee_point_hash;
}

func validator_commitment{range_check_ptr, poseidon_ptr: PoseidonBuiltin*}(
    point: G1Point
) -> (commitment: felt) {
    alloc_locals;

    let (chunks: felt*) = alloc();
    assert [chunks] = point.x.d3;
    assert [chunks + 1] = point.x.d2;
    assert [chunks + 2] = point.x.d1;
    assert [chunks + 3] = point.x.d0;
    assert [chunks + 4] = point.y.d3;
    assert [chunks + 5] = point.y.d2;
    assert [chunks + 6] = point.y.d1;
    assert [chunks + 7] = point.y.d0;

    let (commitment) = poseidon_hash_many(8, chunks);

    return (commitment=commitment);
}
