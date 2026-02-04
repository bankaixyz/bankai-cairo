from cairo.src.light_clients.ethereum.types import EthereumClientOutput, get_ethereum_genesis
from cairo.src.bankai_os.config.types import BankaiOSConfig
from cairo.src.light_clients.ethereum.config.config import get_config

struct BankaiBlock {
    version: felt,
    program_hash: felt,
    block_number: felt,
    ethereum: EthereumClientOutput,
}

func read_block(block_felts: felt*) -> (parsed: BankaiBlock) {
    let block_ptr = cast(block_felts, BankaiBlock*);
    let parsed = [block_ptr];
    return (parsed=parsed);
}

func write_block{output_ptr: felt*}(block: BankaiBlock) {
    // ensure we computed the expected block correctly
    %{ verify_block_result() %}
    // Cast the output buffer to a BankaiBlock pointer
    let block_ptr = cast(output_ptr, BankaiBlock*);
    // Write the block struct into the output buffer
    assert [block_ptr] = block;

    let output_ptr = output_ptr + BankaiBlock.SIZE;
    return ();
}

func get_genesis_block(config: BankaiOSConfig, program_hash: felt) -> (block: BankaiBlock) {
    let eth_config = get_config(config.network_id);
    let (ethereum_genesis) = get_ethereum_genesis(eth_config.genesis_validator_root);
    let block = BankaiBlock(
        version=config.version, program_hash=program_hash, block_number=0, ethereum=ethereum_genesis
    );
    return (block=block);
}
