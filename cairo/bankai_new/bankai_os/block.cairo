from bankai_new.light_clients.ethereum.types import EthereumClientOutput, empty_ethereum_output

struct BankaiBlock {
    version: felt,
    block_number: felt,
    ethereum: EthereumClientOutput,
}

func read_block(block_felts: felt*) -> (parsed: BankaiBlock) {
    let block_ptr = cast(block_felts, BankaiBlock*);
    let parsed = [block_ptr];
    return (parsed=parsed);
}

func write_block{output_ptr: felt*}(block: BankaiBlock) {
    // Cast the output buffer to a BankaiBlock pointer
    let block_ptr = cast(output_ptr, BankaiBlock*);
    // Write the block struct into the output buffer
    assert [block_ptr] = block;
    
    let output_ptr = output_ptr + BankaiBlock.SIZE;
    return ();
}


func get_genesis_block(config: BankaiOSConfig) -> (block: BankaiBlock) {
    let (ethereum_genesis) = get_ethereum_genesis(config);
    let block = BankaiBlock(
        version=BANKAI_VERSION,
        block_number=0,
        ethereum=ethereum,
    );
    return (block=block);
}
