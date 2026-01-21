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

struct CircuitOutput {
    beacon_header_root: Uint256,
    beacon_state_root: Uint256,
    beacon_height: felt,
    n_signers: felt,
    execution_header_root: Uint256,
    execution_header_height: felt,
    current_validator_root: felt,
    next_validator_root: felt,
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
struct CircuitOutput2 {
    block_number: felt,
    beacon: BeaconClientOutput,
    execution: ExecutionClientOutput,
}
