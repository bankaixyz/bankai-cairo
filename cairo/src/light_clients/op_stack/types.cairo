from starkware.cairo.common.uint256 import Uint256

struct OpChainsOutput {
    root: Uint256,
    n_clients: felt,
}

struct OpClientOutput {
    chain_id: felt,
    block_number: felt,
    header_hash: Uint256,
    l1_submission_block: felt,
    mmr_root_keccak: Uint256,
    mmr_root_poseidon: felt,
}

struct OpChainInput {
    client_index: felt,
    prev_merkle_path: Uint256*,
    prev_merkle_path_len: felt,
    prev_output: OpClientOutput,
    output: OpClientOutput,
}

struct OpChainsInput {
    prev_root: Uint256,
    n_updates: felt,
    updates: OpChainInput*,
}
