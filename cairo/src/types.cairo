from definitions import G1Point, G2Point
from starkware.cairo.common.uint256 import Uint256

struct EpochUpdateOutput {
    beacon_header_root: Uint256,
    beacon_state_root: Uint256,
    beacon_height: felt,
    n_signers: felt,
    execution_header_root: Uint256,
    execution_header_height: felt,
    current_committee_hash: Uint256,
}