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

struct SyncCommitteeSignatureInput {
    signature_point: G2Point,
    aggregate_pub: G1Point,
    non_signers: G1Point*,
    n_non_signers: felt,
}

struct ExecutionHeaderProof {
    root: Uint256,
    path: felt**,
    leaf: Uint256,
    index: felt,
    payload_fields: Uint256*,
}

struct SyncCommitteeUpdateInputs {
    path: felt**,
    path_len: felt,
    next_committee_key: UInt384,
    committee_keys_root: felt*,
}

struct ConsensusInputs {
    beacon_header: BeaconHeader,
    signature: SyncCommitteeSignatureInput,
    execution_header_proof: ExecutionHeaderProof,
}

struct CircuitOutput {
    beacon_header_root: Uint256,
    beacon_state_root: Uint256,
    beacon_height: felt,
    n_signers: felt,
    execution_header_root: Uint256,
    execution_header_height: felt,
    current_committee_hash: Uint256,
    next_committee_hash: Uint256,
}
