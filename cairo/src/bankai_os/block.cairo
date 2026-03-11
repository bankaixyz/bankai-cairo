from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.cairo_builtins import BitwiseBuiltin
from starkware.cairo.common.math import split_felt
from starkware.cairo.common.uint256 import Uint256
from cairo.src.light_clients.ethereum.types import EthereumClientOutput, get_ethereum_genesis
from cairo.src.bankai_os.config.types import BankaiOSConfig
from cairo.src.light_clients.ethereum.config.config import get_config
from cairo.src.light_clients.op_stack.lib import get_empty_op_chains_root
from src.core.keccak import keccak_uint256s_bigend
from cairo.src.light_clients.op_stack.types import OpChainsOutput

struct BankaiBlock {
    version: felt,
    program_hash: felt,
    prev_block_hash: Uint256,
    bankai_mmr_root_poseidon: felt,
    bankai_mmr_root_keccak: Uint256,
    block_number: felt,
    ethereum: EthereumClientOutput,
    op_stack: OpChainsOutput,
}

func read_block(block_felts: felt*) -> (parsed: BankaiBlock) {
    let block_ptr = cast(block_felts, BankaiBlock*);
    let parsed = [block_ptr];
    return (parsed=parsed);
}

func felt_to_uint256{range_check_ptr}(value: felt) -> (res: Uint256) {
    let (high, low) = split_felt(value);
    return (res=Uint256(low=low, high=high));
}

func compute_block_hash{range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*}(
    block: BankaiBlock
) -> (hash: Uint256) {
    alloc_locals;
    let (fields_felt_ptr: felt*) = alloc();
    let fields = cast(fields_felt_ptr, Uint256*);

    let (version_u256) = felt_to_uint256(value=block.version);
    let (program_hash_u256) = felt_to_uint256(value=block.program_hash);
    let (bankai_mmr_root_poseidon_u256) = felt_to_uint256(value=block.bankai_mmr_root_poseidon);
    let (block_number_u256) = felt_to_uint256(value=block.block_number);

    let (beacon_slot_number_u256) = felt_to_uint256(value=block.ethereum.beacon.slot_number);
    let (beacon_justified_height_u256) = felt_to_uint256(value=block.ethereum.beacon.justified_height);
    let (beacon_finalized_height_u256) = felt_to_uint256(value=block.ethereum.beacon.finalized_height);
    let (beacon_num_signers_u256) = felt_to_uint256(value=block.ethereum.beacon.num_signers);
    let (beacon_mmr_root_poseidon_u256) = felt_to_uint256(
        value=block.ethereum.beacon.mmr_root_poseidon
    );
    let (beacon_current_validator_root_u256) = felt_to_uint256(
        value=block.ethereum.beacon.current_validator_root
    );
    let (beacon_next_validator_root_u256) = felt_to_uint256(
        value=block.ethereum.beacon.next_validator_root
    );

    let (execution_block_number_u256) = felt_to_uint256(value=block.ethereum.execution.block_number);
    let (execution_justified_height_u256) = felt_to_uint256(
        value=block.ethereum.execution.justified_height
    );
    let (execution_finalized_height_u256) = felt_to_uint256(
        value=block.ethereum.execution.finalized_height
    );
    let (execution_mmr_root_poseidon_u256) = felt_to_uint256(
        value=block.ethereum.execution.mmr_root_poseidon
    );
    let (op_stack_n_clients_u256) = felt_to_uint256(value=block.op_stack.n_clients);

    assert fields[0] = version_u256;
    assert fields[1] = program_hash_u256;
    assert fields[2] = block.prev_block_hash;
    assert fields[3] = bankai_mmr_root_poseidon_u256;
    assert fields[4] = block.bankai_mmr_root_keccak;
    assert fields[5] = block_number_u256;

    assert fields[6] = beacon_slot_number_u256;
    assert fields[7] = block.ethereum.beacon.header_root;
    assert fields[8] = block.ethereum.beacon.state_root;
    assert fields[9] = beacon_justified_height_u256;
    assert fields[10] = beacon_finalized_height_u256;
    assert fields[11] = beacon_num_signers_u256;
    assert fields[12] = block.ethereum.beacon.mmr_root_keccak;
    assert fields[13] = beacon_mmr_root_poseidon_u256;
    assert fields[14] = beacon_current_validator_root_u256;
    assert fields[15] = beacon_next_validator_root_u256;

    assert fields[16] = execution_block_number_u256;
    assert fields[17] = block.ethereum.execution.header_hash;
    assert fields[18] = execution_justified_height_u256;
    assert fields[19] = execution_finalized_height_u256;
    assert fields[20] = block.ethereum.execution.mmr_root_keccak;
    assert fields[21] = execution_mmr_root_poseidon_u256;
    assert fields[22] = block.op_stack.root;
    assert fields[23] = op_stack_n_clients_u256;

    let (hash) = keccak_uint256s_bigend(n_leafs=24, leafs=fields);
    return (hash=hash);
}

func write_block{
    output_ptr: felt*, range_check_ptr, bitwise_ptr: BitwiseBuiltin*, keccak_ptr: felt*
}(block: BankaiBlock) {
    let (block_hash) = compute_block_hash(block=block);
    // ensure we computed the expected block correctly
    %{ verify_block_result() %}
    let block_hash_ptr = cast(output_ptr, Uint256*);
    assert [block_hash_ptr] = block_hash;

    let output_ptr = output_ptr + Uint256.SIZE;
    return ();
}

func get_genesis_block(config: BankaiOSConfig, program_hash: felt) -> (block: BankaiBlock) {
    let eth_config = get_config(config.network_id);
    let (ethereum_genesis) = get_ethereum_genesis(eth_config.genesis_validator_root);
    let zero_u256 = Uint256(low=0, high=0);
    let (op_stack_root) = get_empty_op_chains_root();
    let block = BankaiBlock(
        version=config.version,
        program_hash=program_hash,
        prev_block_hash=zero_u256,
        bankai_mmr_root_poseidon=0,
        bankai_mmr_root_keccak=zero_u256,
        block_number=0,
        ethereum=ethereum_genesis,
        op_stack=OpChainsOutput(root=op_stack_root, n_clients=0),
    );
    return (block=block);
}
