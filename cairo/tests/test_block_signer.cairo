%builtins range_check bitwise poseidon range_check96 add_mod mul_mod

from starkware.cairo.common.cairo_builtins import BitwiseBuiltin, ModBuiltin, PoseidonBuiltin
from definitions import G1Point
from ec_ops import add_ec_points
from sha import SHA256
from cairo.src.utils.utils import pow2alloc128
from cairo.src.io import SyncCommitteeSignerInput
from cairo.src.bls.signer import generate_block_signer_pub

func main{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
}() {
    alloc_locals;

    let (sha256_ptr, sha256_ptr_start) = SHA256.init();
    let (pow2_array) = pow2alloc128();

    with pow2_array, sha256_ptr {
        run_fixtures();
    }

    SHA256.finalize(sha256_ptr_start, sha256_ptr);
    return ();
}

func run_fixtures{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}() {
    alloc_locals;

    local n_fixtures: felt;
    %{ write_block_signer_fixtures_len() %}

    run_fixture_inner(0, n_fixtures);
    return ();
}

func run_fixture_inner{
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    sha256_ptr: felt*,
    pow2_array: felt*,
}(index: felt, n_fixtures: felt) {
    alloc_locals;

    if (index == n_fixtures) {
        return ();
    }

    local signer_data: SyncCommitteeSignerInput;
    %{ write_block_signer_fixture() %}

    let (agg_pub) = generate_block_signer_pub(signer_data);
    let (expected) = aggregate_signers_linear(signer_data.signers, signer_data.n_signers);
    assert_g1_eq(agg_pub, expected);

    return run_fixture_inner(index + 1, n_fixtures);
}

func aggregate_signers_linear{
    range_check_ptr, range_check96_ptr: felt*, add_mod_ptr: ModBuiltin*, mul_mod_ptr: ModBuiltin*
}(signers: G1Point*, n_signers: felt) -> (res: G1Point) {
    if (n_signers == 1) {
        return (signers[0],);
    }

    let (tail_res) = aggregate_signers_linear(signers + G1Point.SIZE, n_signers - 1);
    return add_ec_points(1, tail_res, signers[0]);
}

func assert_g1_eq(a: G1Point, b: G1Point) {
    assert a.x.d0 = b.x.d0;
    assert a.x.d1 = b.x.d1;
    assert a.x.d2 = b.x.d2;
    assert a.x.d3 = b.x.d3;
    assert a.y.d0 = b.y.d0;
    assert a.y.d1 = b.y.d1;
    assert a.y.d2 = b.y.d2;
    assert a.y.d3 = b.y.d3;
    return ();
}
