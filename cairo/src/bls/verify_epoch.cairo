from starkware.cairo.common.cairo_builtins import PoseidonBuiltin, ModBuiltin, BitwiseBuiltin
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.uint256 import Uint256
from definitions import bn, bls, UInt384, one_E12D, N_LIMBS, BASE, E12D, G1Point, G2Point, G1G2Pair
from sha import SHA256
from debug import print_string, print_felt_hex, print_felt
from bls12_381.multi_pairing_check_2 import multi_pairing_check_2P
from hash_to_curve import hash_to_curve

from cairo.src.utils.ssz import SSZ, MerkleTree, MerkleUtils
from cairo.src.utils.constants import g1_negative
from cairo.src.utils.domain import Domain, Network
from cairo.src.bls.signer import (
    aggregate_signer_pubs,
)
from cairo.src.utils.utils import pow2alloc128
from cairo.src.types import SignerData, ExecutionHeaderProof, BeaconHeader, EpochUpdate, EpochUpdateOutput

func run_epoch_update{
    output_ptr: felt*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*,
    sha256_ptr: felt*,
}(epoch_update: EpochUpdate) -> (output: EpochUpdateOutput) {
    alloc_locals;

    // 1. Hash beacon header
    let (header_root, body_root, state_root) = hash_header(epoch_update.header);

    // 2. Compute signing root (this is what validators sign)
    let signing_root = Domain.compute_signing_root(Network.SEPOLIA, header_root, epoch_update.header.slot.low);

    // 3. Hash to curve to get message point
    let (msg_point) = hash_to_curve(1, signing_root);

    // 4. Aggregate signer to get aggregate key that was used to sign the message
    let (committee_hash, agg_key, n_non_signers) = aggregate_signer_pubs(epoch_update.signer_data);
    let n_signers = 512 - n_non_signers;

    // 5. Verify signature
    verify_signature(agg_key, msg_point, epoch_update.sig_point);

    // 6. Hash execution payload root (SSZ encoded execution payload) which is stored in the beacon state
    let (execution_root, execution_hash, execution_height) = SSZ.hash_execution_payload_header_root(epoch_update.execution_header_proof.payload_fields);

    // 7. Verify ssz inclusion proof
    let root_felts = MerkleUtils.chunk_uint256(execution_root);
    let computed_body_root = MerkleTree.hash_merkle_path(
        path=epoch_update.execution_header_proof.path, path_len=4, leaf=root_felts, index=9
    );

    // 8. Assert that the computed body root matches the body root of the verified header
    assert computed_body_root.low = body_root.low;
    assert computed_body_root.high = body_root.high;

    let output = EpochUpdateOutput(
        beacon_header_root=header_root,
        beacon_state_root=state_root,
        beacon_height=epoch_update.header.slot.low,
        n_signers=n_signers,
        execution_header_root=execution_hash,
        execution_header_height=execution_height,
        current_committee_hash=committee_hash,
    );

    return (output=output);
}

func hash_header{
    range_check_ptr, bitwise_ptr: BitwiseBuiltin*, pow2_array: felt*, sha256_ptr: felt*
}(header: BeaconHeader) -> (header_root: Uint256, body_root: Uint256, state_root: Uint256) {
    alloc_locals;

    let header_root = SSZ.hash_header_root(
        header.slot, header.proposer_index, header.parent_root, header.state_root, header.body_root
    );

    return (header_root=header_root, body_root=header.body_root, state_root=header.state_root);
}

func verify_signature{
    range_check_ptr,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
}(agg_pub: G1Point, msg_point: G2Point, sig_point: G2Point) {
    let neg_g1: G1Point = g1_negative();
    let g1_sig_pair: G1G2Pair = G1G2Pair(P=neg_g1, Q=sig_point);
    let pk_msg_pair: G1G2Pair = G1G2Pair(P=agg_pub, Q=msg_point);

    let (inputs: G1G2Pair*) = alloc();
    assert inputs[0] = g1_sig_pair;
    assert inputs[1] = pk_msg_pair;

    // We check the pairs are on the curve in the pairing function
    multi_pairing_check_2P(inputs);
    return ();
}
