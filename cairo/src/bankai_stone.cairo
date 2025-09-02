%builtins output pedersen range_check bitwise poseidon range_check96 add_mod mul_mod

from starkware.cairo.common.cairo_builtins import (
    PoseidonBuiltin,
    ModBuiltin,
    BitwiseBuiltin,
    HashBuiltin,
)
from starkware.cairo.stark_verifier.core.stark import StarkProof
from starkware.cairo.common.uint256 import Uint256
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.builtin_poseidon.poseidon import poseidon_hash_many
from starkware.cairo.common.alloc import alloc
from definitions import UInt384

from cairo.src.recursion.stone import verify_stone_proof

from sha import SHA256
from cairo.src.debug.print import debug_felt_hex, debug_felt, debug_string, info_felt_hex, info_felt, info_string
from cairo.src.io import ConsensusInputs, CircuitOutput, SyncCommitteeUpdateInputs, CircuitOutput2, BeaconClientOutput, ExecutionClientOutput
from cairo.src.types import EpochUpdateOutput
from cairo.src.bls.verify_epoch import run_beacon_update, run_execution_update
from cairo.src.bls.committee_update import run_committee_update
from cairo.src.recursion.proof_output import compute_output_hash
from cairo.src.utils.utils import felt_divmod, pow2alloc128

const BOOTLOADER_PROGRAM_HASH = 0x5AB580B04E3532B6B18F81CFA654A05E29DD8E2352D88DF1E765A84072DB07;
const SYNC_COMMITTEE_PERIOD = 8192;

func main{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
}() {
    alloc_locals;

    debug_string('main');

    let (sha256_ptr, sha256_ptr_start) = SHA256.init();
    let (pow2_array) = pow2alloc128();

    local consensus_inputs: ConsensusInputs;
    local is_genesis: felt;
    local is_committee_update: felt;  // do we add a new committee? 1 if yes, 0 if no
    local program_hash: felt;
    %{ write_consensus_inputs() %}

    debug_string('wrote epoch');

    debug_string('is_genesis');
    debug_felt_hex(is_genesis);
    debug_string('is_committee_update');
    debug_felt_hex(is_committee_update);
    debug_string('program_hash');
    debug_felt_hex(program_hash);

    if (is_genesis == 1) {
        with pow2_array, sha256_ptr {
            let (circuit_output) = handle_genesis_case(consensus_inputs);
        }
        assert is_committee_update = 0;

        write_circuit_output(circuit_output);

        SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);

        return ();
    } else {
        debug_string('recursive case');

        let (_, remainder) = felt_divmod(
            consensus_inputs.beacon_header.slot.low + 1, SYNC_COMMITTEE_PERIOD
        );
        local is_committee_transition: felt;
        if (remainder == 0) {
            is_committee_transition = 1;
        } else {
            is_committee_transition = 0;
        }
        debug_string('is_committee_transition');
        debug_felt_hex(is_committee_transition);

        with pow2_array, sha256_ptr {
            let (circuit_output) = handle_recursive_case(
                consensus_inputs, program_hash, is_committee_transition
            );
        }
        debug_string('confirmed epoch');

        if (is_committee_update == 1) {
            debug_string('committee update');
            // sanity check: next_committee_hash should be 0x0 if we update
            assert circuit_output.beacon.next_committee_hash.low = 0x0;
            assert circuit_output.beacon.next_committee_hash.high = 0x0;

            local committee_input: SyncCommitteeUpdateInputs;

            %{ write_committee_update_inputs() %}
            with pow2_array, sha256_ptr {
                let (state_root, new_next_committee_hash) = run_committee_update(
                    committee_input=committee_input, slot=circuit_output.beacon.slot_number
                );
            }
            debug_string('committee update done');

            // Ensure a valid state root is used to decommit new next_committee_hash
            assert circuit_output.beacon.state_root.low = state_root.low;
            assert circuit_output.beacon.state_root.high = state_root.high;
            write_circuit_output(circuit_output);

            SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);
            return ();
        } else {
            debug_string('no committee update');
            write_circuit_output(circuit_output);

            SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);
            return ();
        }
    }
}

func handle_recursive_case{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}(consensus_inputs: ConsensusInputs, program_hash: felt, is_committee_transition: felt) -> (circuit_output: CircuitOutput2) {
    alloc_locals;

    debug_string('handle_recursive_case');

    local previous_output: CircuitOutput2;
    %{ load_previous_output() %}

    let (expected_output_hash) = compute_output_hash(program_hash, previous_output);

    info_string('expected output hash');
    info_felt_hex(expected_output_hash);

    %{ write_stone_proof_inputs() %}
    let (proof_program_hash, output_hash) = verify_stone_proof();

    debug_string('program hash');
    debug_felt_hex(program_hash);


    info_string('output hash');
    info_felt_hex(output_hash);

    info_string('proof program hash');
    info_felt_hex(proof_program_hash);

    // Ensure the proof contains the expected values
    assert output_hash = expected_output_hash;
    assert proof_program_hash = BOOTLOADER_PROGRAM_HASH;

    info_string('verified stone proof');

    info_string('ran beacon update');
    let (
       beacon_client_output, body_root
    ) = run_beacon_update(consensus_inputs, is_committee_transition, previous_output);

    info_string('ran beacon update');

    let (execution_client_output) = run_execution_update(body_root, consensus_inputs.execution_header_proof, previous_output);

    let circuit_output = CircuitOutput2(
        block_number=previous_output.block_number + 1,
        beacon=beacon_client_output,
        execution=execution_client_output,
    );

    return (circuit_output=circuit_output);
}

func handle_genesis_case{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}(consensus_inputs: ConsensusInputs) -> (circuit_output: CircuitOutput2) {
    alloc_locals;

    // For the genesis case, I hardcode the previous proof values.
    tempvar expected_genesis_committee = Uint256(
        low=0x36c253a239c2878d1a6aa8d46dfe4be8, high=0xdfb8eb2acda46f413d93538c7d6b3610
    );

    let genesis_output = CircuitOutput2(
        block_number=0,
        beacon=BeaconClientOutput(
            slot_number=0,
            header_root=Uint256(low=0x0, high=0x0),
            state_root=Uint256(low=0x0, high=0x0),
            justified_height=0,
            finalized_height=0,
            num_signers=0,
            mmr_root_sha=Uint256(low=0x0, high=0x0),
            mmr_root_poseidon=0,
            current_committee_hash=expected_genesis_committee, // important
            next_committee_hash=Uint256(low=0x0, high=0x0),
        ),
        execution=ExecutionClientOutput(
            block_number=0,
            header_hash=Uint256(low=0x0, high=0x0),
            justified_height=0,
            finalized_height=0,
            mmr_root_sha=Uint256(low=0x0, high=0x0),
            mmr_root_poseidon=0,
        ),
    );

    let (
       beacon_client_output, body_root
    ) = run_beacon_update(consensus_inputs, 0, genesis_output);

    info_string('ran beacon update');

    let (execution_client_output) = run_execution_update(body_root, consensus_inputs.execution_header_proof, genesis_output);

    let circuit_output = CircuitOutput2(
        block_number=1,
        beacon=beacon_client_output,
        execution=execution_client_output,
    );

    return (circuit_output=circuit_output);
}

func write_circuit_output{output_ptr: felt*, range_check_ptr}(
    output: CircuitOutput2
) {
    assert [output_ptr] = output.block_number;
    assert [output_ptr + 1] = output.beacon.slot_number;
    assert [output_ptr + 2] = output.beacon.header_root.low;
    assert [output_ptr + 3] = output.beacon.header_root.high;
    assert [output_ptr + 4] = output.beacon.justified_height;
    assert [output_ptr + 5] = output.beacon.finalized_height;
    assert [output_ptr + 6] = output.beacon.num_signers;
    assert [output_ptr + 7] = output.beacon.mmr_root_sha.low;
    assert [output_ptr + 8] = output.beacon.mmr_root_sha.high;
    assert [output_ptr + 9] = output.beacon.mmr_root_poseidon;
    assert [output_ptr + 10] = output.beacon.current_committee_hash.low;
    assert [output_ptr + 11] = output.beacon.current_committee_hash.high;
    assert [output_ptr + 12] = output.beacon.next_committee_hash.low;
    assert [output_ptr + 13] = output.beacon.next_committee_hash.high;
    assert [output_ptr + 14] = output.execution.block_number;
    assert [output_ptr + 15] = output.execution.header_hash.low;
    assert [output_ptr + 16] = output.execution.header_hash.high;
    assert [output_ptr + 17] = output.execution.justified_height;
    assert [output_ptr + 18] = output.execution.finalized_height;
    assert [output_ptr + 19] = output.execution.mmr_root_sha.low;
    assert [output_ptr + 20] = output.execution.mmr_root_sha.high;
    assert [output_ptr + 21] = output.execution.mmr_root_poseidon;

    let output_ptr = output_ptr + 22;
    return ();
}
