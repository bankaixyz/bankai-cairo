from starkware.cairo.common.cairo_builtins import (
    PoseidonBuiltin,
    ModBuiltin,
    BitwiseBuiltin,
    KeccakBuiltin,
)
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.uint256 import Uint256
from definitions import bn, bls, UInt384, one_E12D, N_LIMBS, BASE, E12D, G1Point, G2Point, G1G2Pair
from sha import SHA256
from debug import print_string, print_felt_hex, print_felt
from bls12_381.multi_pairing_check_2 import multi_pairing_check_2P
from hash_to_curve import hash_to_curve
from cairo.src.debug.print import (
    info_string,
    info_uint256,
    debug_string,
    debug_uint256,
    debug_felt_hex,
    debug_felt,
)
from cairo.src.light_clients.ethereum.utils.ssz import SSZ, MerkleTree, MerkleUtils
from cairo.src.light_clients.ethereum.utils.constants import g1_negative
from cairo.src.light_clients.ethereum.config.types import Networks
from cairo.src.light_clients.ethereum.utils.domain import Fork, Domain
from cairo.src.light_clients.ethereum.bls.signer import generate_block_signer_pub
from cairo.src.utils.utils import pow2alloc128
from cairo.src.light_clients.ethereum.types import (
    ExecutionHeaderProof,
    BeaconHeader,
    ConsensusInputs,
    BeaconClientOutput,
    ExecutionClientOutput,
    EthereumClientOutput,
)
from src.beacon.lib import run_beacon_mmr_update
from src.execution.lib import run_execution_mmr_update

func run_beacon_update{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*,
    sha256_ptr: felt*,
}(consensus_inputs: ConsensusInputs, is_committee_transition: felt, prev: BeaconClientOutput) -> (
    output: BeaconClientOutput, body_root: Uint256
) {
    alloc_locals;

    // 1. Hash beacon header
    let (header_root, body_root, state_root) = hash_header(consensus_inputs.beacon_header);

    // 2. Compute signing root (this is what validators sign)
    let signing_root = Domain.compute_signing_root(
        Networks.SEPOLIA, header_root, consensus_inputs.beacon_header.slot.low
    );

    // 3. Hash to curve to get message point
    let (msg_point) = hash_to_curve(1, signing_root);

    // 4. Aggregate signer to get aggregate key that was used to sign the message
    let (agg_key) = generate_block_signer_pub(consensus_inputs.signature);
    let n_signers = consensus_inputs.signature.n_signers;
    let validator_root = consensus_inputs.signature.validator_root;

    // 5. Verify signature
    verify_signature(agg_key, msg_point, consensus_inputs.signature_point);
    debug_string('beacon: signature verified');

    local current_validator_root: felt;
    local next_validator_root: felt;

    // Assert the correct validator root is used, in case of committee transition
    if (is_committee_transition == 1) {
        assert prev.next_validator_root = validator_root;

        // In transition, move the keys from the previous committee to the next committee
        assert current_validator_root = prev.next_validator_root;
        assert next_validator_root = 0x0;
    } else {
        assert prev.current_validator_root = validator_root;

        // In non-transition, use the current validator root
        assert current_validator_root = prev.current_validator_root;
        assert next_validator_root = prev.next_validator_root;
    }

    let (
        new_keccak_root, new_poseidon_root, new_mmr_size, last_header_root
    ) = run_beacon_mmr_update();

    // Ensure the MMR root corresponds to the header verified via BLS
    assert last_header_root.low = header_root.low;
    assert last_header_root.high = header_root.high;

    debug_string('beacon: validator roots set');
    let output = BeaconClientOutput(
        slot_number=consensus_inputs.beacon_header.slot.low,
        header_root=header_root,
        state_root=state_root,
        justified_height=prev.slot_number,
        finalized_height=prev.justified_height,
        num_signers=n_signers,
        mmr_root_keccak=new_keccak_root,
        mmr_root_poseidon=new_poseidon_root,
        current_validator_root=current_validator_root,
        next_validator_root=next_validator_root,
    );

    return (output=output, body_root=body_root);
}

func run_execution_update{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*,
    sha256_ptr: felt*,
}(
    body_root: Uint256, execution_header_proof: ExecutionHeaderProof, prev: ExecutionClientOutput
) -> (output: ExecutionClientOutput) {
    alloc_locals;

    // 1. Hash execution payload root (SSZ encoded execution payload) which is stored in the beacon state
    let (execution_root, header_hash, block_number) = SSZ.hash_execution_payload_header_root(
        execution_header_proof.payload_fields
    );

    // 2. Verify ssz inclusion proof
    let root_felts = MerkleUtils.chunk_uint256(execution_root);
    let computed_body_root = MerkleTree.hash_merkle_path(
        path=execution_header_proof.path, path_len=4, leaf=root_felts, index=9
    );

    // 3. Assert that the computed body root matches the body root of the verified header
    assert computed_body_root.low = body_root.low;
    assert computed_body_root.high = body_root.high;

    // 4. Update the MMR
    let (
        new_keccak_root, new_poseidon_root, new_mmr_size, last_header_hash
    ) = run_execution_mmr_update();

    // 5. Assert that the last header hash matches the header hash
    assert last_header_hash.low = header_hash.low;
    assert last_header_hash.high = header_hash.high;

    let output = ExecutionClientOutput(
        block_number=block_number,
        header_hash=header_hash,
        justified_height=prev.block_number,
        finalized_height=prev.justified_height,
        mmr_root_keccak=new_keccak_root,
        mmr_root_poseidon=new_poseidon_root,
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
