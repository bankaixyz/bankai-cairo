from cairo.bankai_new.bankai_os.block import BankaiBlock

struct MockRecursionInputs {
    program_hash: felt,
}

func mock_verify_proof(block: BankaiBlock) -> (program_hash: felt) {
    alloc_locals;
    assert 1 = 1;

    local recursion: MockRecursionInputs;
    %{ write_mock_recursion_inputs() %}

    assert recursion.program_hash = block.program_hash;

    return (program_hash=recursion.program_hash);

}
