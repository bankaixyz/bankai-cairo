from definitions import G1Point, G2Point
from starkware.cairo.common.uint256 import Uint256
from definitions import UInt384

struct BeaconHeader {
    slot: Uint256,
    proposer_index: Uint256,
    parent_root: Uint256,
    state_root: Uint256,
    body_root: Uint256,
}

struct SyncCommitteeSignerInput {
    validator_root: felt,
    signers: G1Point*,
    indexes: felt*,
    proofs: felt**,
    proofs_len: felt,
    n_signers: felt,
}

struct ExecutionHeaderProof {
    root: Uint256,
    path: felt**,
    leaf: Uint256,
    index: felt,
    payload_fields: Uint256*,
}

struct SyncCommitteeUpdateInputs {
    slot: felt,
    path: felt**,
    path_len: felt,
    aggregate_committee_key: UInt384,
    validator_pubs: UInt384*,
    committee_keys_root: felt*,
}

struct ConsensusInputs {
    beacon_header: BeaconHeader,
    signature_point: G2Point,
    signature: SyncCommitteeSignerInput,
    execution_header_proof: ExecutionHeaderProof,
}


struct BeaconClientOutput {
    slot_number: felt,
    header_root: Uint256,
    state_root: Uint256,
    justified_height: felt,
    finalized_height: felt,
    num_signers: felt,
    mmr_root_keccak: Uint256,
    mmr_root_poseidon: felt,
    current_validator_root: felt,
    next_validator_root: felt,
}

struct ExecutionClientOutput {
    block_number: felt,
    header_hash: Uint256,
    justified_height: felt,
    finalized_height: felt,
    mmr_root_keccak: Uint256,
    mmr_root_poseidon: felt,
}

struct EthereumClientOutput {
    beacon: BeaconClientOutput,
    execution: ExecutionClientOutput,
}

func get_ethereum_genesis(current_validator_root: felt) -> (out: EthereumClientOutput) {
    let zero_u256 = Uint256(low=0, high=0);
    let beacon = BeaconClientOutput(
        slot_number=0,
        header_root=zero_u256,
        state_root=zero_u256,
        justified_height=0,
        finalized_height=0,
        num_signers=0,
        mmr_root_keccak=zero_u256,
        mmr_root_poseidon=0,
        current_validator_root=current_validator_root,
        next_validator_root=0,
    );
    let execution = ExecutionClientOutput(
        block_number=0,
        header_hash=zero_u256,
        justified_height=0,
        finalized_height=0,
        mmr_root_keccak=zero_u256,
        mmr_root_poseidon=0,
    );
    let out = EthereumClientOutput(beacon=beacon, execution=execution);
    return (out=out);
}