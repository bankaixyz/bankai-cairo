from starkware.cairo.common.cairo_builtins import (
    PoseidonBuiltin,
    ModBuiltin,
    BitwiseBuiltin,
    HashBuiltin,
)

from bankai_new.bankai_os.block import BankaiBlock, write_block
from bankai_new.light_clients.ethereum.types import empty_ethereum_output
from bankai_new.light_clients.ethereum.lib import run as run_ethereum
from bankai_new.bankai_os.config.config import get_config, EthereumNetwork

const BANKAI_VERSION = 1;

func run_bankai_os{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}() {
    alloc_locals;

    let config = get_config(network_id=EthereumNetwork.TESTNET);

    // ToDo: Add new hint here
    local prev: BankaiBlock;
    local is_genesis: felt;
    local program_hash: felt;
    %{ write_prev_block() %}

    if (is_genesis == 1) {
        let (block) = handle_genesis_case(config);

        write_block(block=block);
        return ();

    } else {
        let (block) = handle_recursive_case(prev);
    }

    // TODO: read inputs (is_genesis, program_hash, prev output) via hints
    let (block) = handle_genesis_case();
    write_block(block=block);

    return ();
}

func handle_genesis_case(config: BankaiOSConfig) -> (block: BankaiBlock) {
    // Get the genesis Bankai block
    let (prev) = get_genesis_block(config);




    return (block=block);
}

func handle_recursive_case(program_hash: felt) -> (block: BankaiBlock) {
    // TODO: load previous output, verify proof, run clients, build block
    let (prev) = empty_ethereum_output();
    let (ethereum) = run_ethereum(prev);
    let block = BankaiBlock(
        version=BANKAI_VERSION,
        block_number=0,
        ethereum=ethereum,
    );
    return (block=block);
}
