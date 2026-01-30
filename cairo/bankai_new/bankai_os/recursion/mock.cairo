from cairo.bankai_new.bankai_os.block import BankaiBlock

func mock_verify_proof(block: BankaiBlock) -> (program_hash: felt) {
    assert 1 = 1;

    return (program_hash=block.program_hash);

}