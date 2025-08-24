%builtins output pedersen range_check bitwise poseidon range_check96 add_mod mul_mod

from starkware.cairo.common.cairo_builtins import PoseidonBuiltin, ModBuiltin, BitwiseBuiltin, HashBuiltin
from starkware.cairo.stark_verifier.core.stark import StarkProof
from starkware.cairo.common.uint256 import Uint256
from starkware.cairo.common.memcpy import memcpy
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.builtin_poseidon.poseidon import poseidon_hash_many
from starkware.cairo.common.alloc import alloc
from definitions import UInt384

from cairo.src.recursion.stone import verify_stone_proof

from sha import SHA256
from debug import print_felt_hex, print_string
from cairo.src.types import EpochUpdate, EpochUpdateOutput, CircuitOutput
from cairo.src.bls.verify_epoch import run_epoch_update
from cairo.src.bls.committee_update import run_committee_update
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

    let (sha256_ptr, sha256_ptr_start) = SHA256.init();
    let (pow2_array) = pow2alloc128();

    local epoch_update: EpochUpdate;
    local is_genesis: felt;
    local is_committee_update: felt; // do we add a new committee? 1 if yes, 0 if no
    local program_hash: felt;
    %{ write_epoch_update_inputs() %}

    if (is_genesis == 1) {
        with pow2_array, sha256_ptr {
            let (epoch_update_output) = handle_genesis_case(epoch_update);
        }
        let next_committee_hash = Uint256(low=0x0, high=0x0);
        assert is_committee_update = 0;
        write_circuit_output(epoch_output=epoch_update_output, next_committee_hash=next_committee_hash, is_committee_transition=0);

        SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);

        return ();
    } else {
        print_string('recursive case');

        let (_, remainder) = felt_divmod(epoch_update.header.slot.low + 1, SYNC_COMMITTEE_PERIOD);
        local is_committee_transition: felt;
        if (remainder == 0) {
            is_committee_transition = 1;
        } else {
            is_committee_transition = 0;
        }
        print_string('is_committee_transition');
        print_felt_hex(is_committee_transition);

        with pow2_array, sha256_ptr {
            let (epoch_update_output, next_committee_hash) = handle_recursive_case(epoch_update, program_hash, is_committee_transition);
        }
        print_string('confirmed epoch');

        if (is_committee_update == 1) {
            print_string('committee update');
            // sanity check: next_committee_hash should be 0x0 if we update
            assert next_committee_hash.low = 0x0;
            assert next_committee_hash.high = 0x0;

            let (committee_keys_root: felt*) = alloc();
            let (path: felt**) = alloc();
            local path_len: felt;
            local aggregate_committee_key: UInt384;
            
            %{ write_committee_update_inputs() %}
            with pow2_array, sha256_ptr {
                let (state_root, new_next_committee_hash) = run_committee_update(
                    committee_keys_root=committee_keys_root,
                    path=path,
                    path_len=path_len,
                    aggregate_committee_key=aggregate_committee_key,
                    slot=epoch_update_output.beacon_height
                );
            }
            print_string('committee update done');

            // Ensure a valid state root is used to decommit new next_committee_hash
            assert epoch_update_output.beacon_state_root.low = state_root.low;
            assert epoch_update_output.beacon_state_root.high = state_root.high;
            write_circuit_output(epoch_output=epoch_update_output, next_committee_hash=new_next_committee_hash, is_committee_transition=is_committee_transition);

            SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);
            return ();
        } else {
            print_string('no committee update');
            write_circuit_output(epoch_output=epoch_update_output, next_committee_hash=next_committee_hash, is_committee_transition=is_committee_transition);
            
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
}(epoch_update: EpochUpdate, program_hash: felt, is_committee_transition: felt) -> (EpochUpdateOutput, Uint256) {
    alloc_locals;

    let (epoch_update_output) = run_epoch_update(epoch_update);
    print_string('epoch update output');

    local expected_proof_output: CircuitOutput;
    %{ load_expected_proof_output() %}


    // Check that expected matches the committee hash that was used to sign

    if (is_committee_transition == 1) {
        print_string('exp com');
        print_felt_hex(expected_proof_output.next_committee_hash.low);
        print_felt_hex(expected_proof_output.next_committee_hash.high);

        print_string('epoch update output');
        print_felt_hex(epoch_update_output.current_committee_hash.low);
        print_felt_hex(epoch_update_output.current_committee_hash.high);
        assert expected_proof_output.next_committee_hash.low = epoch_update_output.current_committee_hash.low;
        assert expected_proof_output.next_committee_hash.high = epoch_update_output.current_committee_hash.high;
    } else {
        print_string('exp com');
        print_felt_hex(expected_proof_output.current_committee_hash.low);
        print_felt_hex(expected_proof_output.current_committee_hash.high);

        print_string('epoch update output');
        print_felt_hex(epoch_update_output.current_committee_hash.low);
        print_felt_hex(epoch_update_output.current_committee_hash.high);
        assert expected_proof_output.current_committee_hash.low = epoch_update_output.current_committee_hash.low;
        assert expected_proof_output.current_committee_hash.high = epoch_update_output.current_committee_hash.high;
    }

    print_string('checked committee hash');

    print_string('program hash');
    print_felt_hex(program_hash);

    print_felt_hex(expected_proof_output.beacon_header_root.low);
    print_felt_hex(expected_proof_output.beacon_header_root.high);
    print_felt_hex(expected_proof_output.beacon_state_root.low);
    print_felt_hex(expected_proof_output.beacon_state_root.high);
    print_felt_hex(expected_proof_output.beacon_height);
    print_felt_hex(expected_proof_output.n_signers);
    print_felt_hex(expected_proof_output.execution_header_root.low);
    print_felt_hex(expected_proof_output.execution_header_root.high);
    print_felt_hex(expected_proof_output.execution_header_height);
    print_felt_hex(expected_proof_output.current_committee_hash.low);
    print_felt_hex(expected_proof_output.current_committee_hash.high);
    print_felt_hex(expected_proof_output.next_committee_hash.low);
    print_felt_hex(expected_proof_output.next_committee_hash.high);

    // Construct the expected verifier output
    tempvar expected_verifier_output = cast(
        new (
            1, 15, program_hash,
            expected_proof_output.beacon_header_root.low,
            expected_proof_output.beacon_header_root.high,
            expected_proof_output.beacon_state_root.low,
            expected_proof_output.beacon_state_root.high,
            expected_proof_output.beacon_height,
            expected_proof_output.n_signers,
            expected_proof_output.execution_header_root.low,
            expected_proof_output.execution_header_root.high,
            expected_proof_output.execution_header_height,
            expected_proof_output.current_committee_hash.low,
            expected_proof_output.current_committee_hash.high,
            expected_proof_output.next_committee_hash.low,
            expected_proof_output.next_committee_hash.high
        ), felt*
    );

    let (expected_output_hash: felt) = poseidon_hash_many(n=16, elements=expected_verifier_output);
    print_string('expected output hash');
    print_felt_hex(expected_output_hash);



    %{ write_stark_proof_inputs() %}
    let (proof_program_hash, output_hash) = verify_stone_proof();

    print_string('output hash');
    print_felt_hex(output_hash);

    print_string('proof program hash');
    print_felt_hex(proof_program_hash);

    // Ensure the proof contains the expected values
    assert output_hash = expected_output_hash;    
    assert proof_program_hash = BOOTLOADER_PROGRAM_HASH;

    return (epoch_update_output, expected_proof_output.next_committee_hash);
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
}(epoch_update: EpochUpdate) -> (epoch_update_output: EpochUpdateOutput) {
    alloc_locals;

    let (epoch_update_output) = run_epoch_update(epoch_update);

    tempvar expected_genesis_committee = Uint256(low=0xe5fec5cd2304cab6086b1eea025ccd74, high=0xf32b83714599ab70193ba4597159560c);
    assert expected_genesis_committee.low = epoch_update_output.current_committee_hash.low;
    assert expected_genesis_committee.high = epoch_update_output.current_committee_hash.high;

    return (epoch_update_output=epoch_update_output);
}

func write_circuit_output{
    output_ptr: felt*,
    range_check_ptr,
}(epoch_output: EpochUpdateOutput, next_committee_hash: Uint256, is_committee_transition: felt) {
    assert [output_ptr] = epoch_output.beacon_header_root.low;
    assert [output_ptr + 1] = epoch_output.beacon_header_root.high;
    assert [output_ptr + 2] = epoch_output.beacon_state_root.low;
    assert [output_ptr + 3] = epoch_output.beacon_state_root.high;
    assert [output_ptr + 4] = epoch_output.beacon_height;
    assert [output_ptr + 5] = epoch_output.n_signers;
    assert [output_ptr + 6] = epoch_output.execution_header_root.low;
    assert [output_ptr + 7] = epoch_output.execution_header_root.high;
    assert [output_ptr + 8] = epoch_output.execution_header_height;

    if (is_committee_transition == 1) {
        print_string('committee update');
        assert [output_ptr + 9] = next_committee_hash.low;
        assert [output_ptr + 10] = next_committee_hash.high;
        assert [output_ptr + 11] = 0x0;
        assert [output_ptr + 12] = 0x0;
        tempvar range_check_ptr = range_check_ptr;
    } else {
        print_string('no committee update');
        assert [output_ptr + 9] = epoch_output.current_committee_hash.low;
        assert [output_ptr + 10] = epoch_output.current_committee_hash.high;
        assert [output_ptr + 11] = next_committee_hash.low;
        assert [output_ptr + 12] = next_committee_hash.high;
        tempvar range_check_ptr = range_check_ptr;
    }

    print_string('output_ptr');
    print_felt_hex(output_ptr[0]);
    print_felt_hex(output_ptr[1]);
    print_felt_hex(output_ptr[2]);
    print_felt_hex(output_ptr[3]);
    print_felt_hex(output_ptr[4]);
    print_felt_hex(output_ptr[5]);
    print_felt_hex(output_ptr[6]);
    print_felt_hex(output_ptr[7]);
    print_felt_hex(output_ptr[8]);
    print_felt_hex(output_ptr[9]);
    print_felt_hex(output_ptr[10]);
    print_felt_hex(output_ptr[11]);
    print_felt_hex(output_ptr[12]);

    let output_ptr = output_ptr + 13;
    return ();
}