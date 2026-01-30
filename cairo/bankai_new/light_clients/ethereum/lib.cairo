from starkware.cairo.common.cairo_builtins import PoseidonBuiltin, ModBuiltin, BitwiseBuiltin
from starkware.cairo.common.uint256 import Uint256
from cairo.bankai_new.debug.print import (
    debug_felt_hex,
    debug_string,
    info_felt,
    info_string,
)
from cairo.bankai_new.utils.utils import felt_divmod, pow2alloc128
from cairo.bankai_new.light_clients.ethereum.types import (
    ConsensusInputs,
    SyncCommitteeUpdateInputs,
    BeaconClientOutput,
    ExecutionClientOutput,
    get_ethereum_genesis
)
from cairo.bankai_new.light_clients.ethereum.types import EthereumClientOutput
from cairo.bankai_new.light_clients.ethereum.bls.verify_epoch import run_beacon_update, run_execution_update
from cairo.bankai_new.light_clients.ethereum.bls.committee_update import run_committee_update
from cairo.bankai_new.light_clients.ethereum.config.config import get_config, EthereumConfig

func run{
    output_ptr: felt*,
    pedersen_ptr: felt*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}(prev: EthereumClientOutput, network_id: felt) -> (output: EthereumClientOutput) {
    alloc_locals;

    let config = get_config(network_id);

    local consensus_inputs: ConsensusInputs;
    local is_genesis: felt;
    local is_committee_update: felt;
    local program_hash: felt;
    %{ write_consensus_inputs() %}

    debug_string('is_genesis');
    debug_felt_hex(is_genesis);
    debug_string('is_committee_update');
    debug_felt_hex(is_committee_update);

    if (is_genesis == 1) {
        info_string('handle_genesis_case');

        let (prev) = get_ethereum_genesis(config.genesis_validator_root);
        with pow2_array, sha256_ptr {
            let (output) = handle_genesis_case(prev, consensus_inputs);
        }
        assert is_committee_update = 0;
        return (output=output);
    } else {
        info_string('handle_recursive_case');
        with pow2_array, sha256_ptr, config {
            let (output) = handle_recursive_case(consensus_inputs, prev);
        }
        info_string('confirmed epoch');

        if (is_committee_update == 1) {
            info_string('committee update');
            // sanity check: next_validator_root should be 0x0 if we update
            assert output.beacon.next_validator_root = 0x0;

            local committee_input: SyncCommitteeUpdateInputs;
            %{ write_committee_update_inputs() %}

            with pow2_array, sha256_ptr {
                let (state_root, new_next_validator_root) = run_committee_update(
                    committee_input=committee_input
                );
            }

            // Ensure a valid state root is used to decommit new next_validator_root
            assert output.beacon.state_root.low = state_root.low;
            assert output.beacon.state_root.high = state_root.high;

            info_string('updated next_validator_root');
            info_felt(new_next_validator_root);

            // Update the next_validator_root in the final output
            let updated_beacon_output = BeaconClientOutput(
                slot_number=output.beacon.slot_number,
                header_root=output.beacon.header_root,
                state_root=output.beacon.state_root,
                justified_height=output.beacon.justified_height,
                finalized_height=output.beacon.finalized_height,
                num_signers=output.beacon.num_signers,
                mmr_root_keccak=output.beacon.mmr_root_keccak,
                mmr_root_poseidon=output.beacon.mmr_root_poseidon,
                current_validator_root=output.beacon.current_validator_root,
                next_validator_root=new_next_validator_root,
            );
            let final_output = EthereumClientOutput(
                beacon=updated_beacon_output, execution=output.execution
            );

            return (output=final_output);
        } else {
            debug_string('no committee update');
            return (output=output);
        }
    }
}

func handle_recursive_case{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
    config: EthereumConfig,
}(
    consensus_inputs: ConsensusInputs, prev: EthereumClientOutput
) -> (output: EthereumClientOutput) {
    alloc_locals;

    let (old_committee_term, _) = felt_divmod(
        prev.beacon.slot_number + 1, config.sync_committee_period
    );

    let (new_committee_term, _) = felt_divmod(
        consensus_inputs.beacon_header.slot.low + 1, config.sync_committee_period
    );

    local is_committee_transition = new_committee_term - old_committee_term;

    debug_string('is_committee_transition');
    debug_felt_hex(is_committee_transition);

    let (beacon_client_output, body_root) = run_beacon_update(
        consensus_inputs, is_committee_transition, prev.beacon
    );

    info_string('ran beacon update');

    let (execution_client_output) = run_execution_update(
        body_root, consensus_inputs.execution_header_proof, prev.execution
    );

    info_string('ran execution update');

    let output = EthereumClientOutput(
        beacon=beacon_client_output, execution=execution_client_output
    );

    return (output=output);
}

func handle_genesis_case{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}(prev: EthereumClientOutput, consensus_inputs: ConsensusInputs) -> (output: EthereumClientOutput) {
    alloc_locals;

    let (beacon_client_output, body_root) = run_beacon_update(consensus_inputs, 0, prev.beacon);

    info_string('ran beacon update');

    let (execution_client_output) = run_execution_update(
        body_root, consensus_inputs.execution_header_proof, prev.execution
    );

    let output = EthereumClientOutput(
        beacon=beacon_client_output, execution=execution_client_output
    );

    return (output=output);
}

