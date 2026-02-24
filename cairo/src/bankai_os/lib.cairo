from starkware.cairo.common.cairo_builtins import (
    PoseidonBuiltin,
    ModBuiltin,
    BitwiseBuiltin,
    HashBuiltin,
)
from starkware.cairo.common.uint256 import Uint256

from cairo.src.bankai_os.block import BankaiBlock, compute_block_hash, write_block, get_genesis_block
from cairo.src.light_clients.ethereum.lib import run as run_ethereum
from cairo.src.bankai_os.config.config import get_config, Networks
from cairo.src.bankai_os.recursion.mock import mock_verify_proof
from cairo.src.bankai_os.config.types import BankaiOSConfig
from cairo.src.light_clients.ethereum.bls.verify_epoch import (
    run_beacon_update,
    run_execution_update,
)
from cairo.src.light_clients.ethereum.types import EthereumClientOutput
from src.bankai.lib import run_bankai_mmr_update

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

    let (config) = get_config(network_id=Networks.TESTNET);

    local is_genesis: felt;
    local program_hash: felt;
    %{ write_init_data() %}  // ToDo: create new hint

    if (is_genesis == 1) {
        let (block) = handle_genesis_case(config, program_hash);

        write_block(block=block);
        return ();
    } else {
        let (block) = handle_recursive_case(config);

        write_block(block=block);
        return ();
    }
}

func handle_genesis_case{
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
}(config: BankaiOSConfig, program_hash: felt) -> (block: BankaiBlock) {
    alloc_locals;
    // Get the genesis Bankai block
    let (prev) = get_genesis_block(config, program_hash);

    // Run Ethereum Light Client
    let (eth_output) = run_ethereum(prev.ethereum, config.network_id, 1);
    let zero_u256 = Uint256(low=0, high=0);

    let block = BankaiBlock(
        version=config.version,
        program_hash=prev.program_hash,
        prev_block_hash=zero_u256,
        bankai_mmr_root_poseidon=0,
        bankai_mmr_root_keccak=zero_u256,
        block_number=1,
        ethereum=eth_output,
        op_chains=prev.op_chains,
    );

    return (block=block);
}

func handle_recursive_case{
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
}(config: BankaiOSConfig) -> (block: BankaiBlock) {
    alloc_locals;

    local prev: BankaiBlock;
    %{ write_previous_block() %}

    // Mock Verify Proof
    let (derived_program_hash) = mock_verify_proof(block=prev);
    // Ensure proof program hash is consistent
    assert derived_program_hash = prev.program_hash;

    let (prev_block_hash) = compute_block_hash(block=prev);
    let (bankai_mmr_root_keccak, bankai_mmr_root_poseidon, _) = run_bankai_mmr_update(
        leaf=prev_block_hash
    );

    // Run Ethereum Light Client
    let (eth_output) = run_ethereum(prev.ethereum, config.network_id, 0);

    let block = BankaiBlock(
        version=config.version,
        program_hash=prev.program_hash,
        prev_block_hash=prev_block_hash,
        bankai_mmr_root_poseidon=bankai_mmr_root_poseidon,
        bankai_mmr_root_keccak=bankai_mmr_root_keccak,
        block_number=prev.block_number + 1,
        ethereum=eth_output,
        op_chains=prev.op_chains,
    );

    return (block=block);
}
