%builtins range_check bitwise poseidon range_check96 add_mod mul_mod

from starkware.cairo.common.cairo_builtins import BitwiseBuiltin, ModBuiltin, PoseidonBuiltin
from definitions import UInt384
from sha import SHA256
from cairo.src.utils.utils import pow2alloc128
from cairo.src.light_clients.ethereum.types import SyncCommitteeUpdateInputs
from cairo.src.light_clients.ethereum.bls.committee_update import run_committee_update

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
    %{ write_committee_update_fixtures_len() %}

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

    local committee_input: SyncCommitteeUpdateInputs;
    local expected_slot: felt;
    local expected_path: felt**;
    local expected_path_len: felt;
    local expected_aggregate_committee_key: UInt384;
    local expected_validator_pubs: UInt384*;
    local expected_validator_pubs_len: felt;
    local expected_committee_keys_root: felt*;
    %{ write_committee_update_fixture() %}

    assert committee_input.slot = expected_slot;
    assert committee_input.path_len = expected_path_len;
    assert_uint384_eq(committee_input.aggregate_committee_key, expected_aggregate_committee_key);
    assert_bits32_eq(committee_input.committee_keys_root, expected_committee_keys_root);
    assert_path_eq(committee_input.path, expected_path, expected_path_len, 0);
    assert_uint384_array_eq(
        committee_input.validator_pubs, expected_validator_pubs, expected_validator_pubs_len, 0
    );

    let (_state_root, _validator_root) = run_committee_update(committee_input=committee_input);

    return run_fixture_inner(index + 1, n_fixtures);
}

func assert_uint384_eq(a: UInt384, b: UInt384) {
    assert a.d0 = b.d0;
    assert a.d1 = b.d1;
    assert a.d2 = b.d2;
    assert a.d3 = b.d3;
    return ();
}

func assert_bits32_eq(a: felt*, b: felt*) {
    assert [a] = [b];
    assert [a + 1] = [b + 1];
    assert [a + 2] = [b + 2];
    assert [a + 3] = [b + 3];
    assert [a + 4] = [b + 4];
    assert [a + 5] = [b + 5];
    assert [a + 6] = [b + 6];
    assert [a + 7] = [b + 7];
    return ();
}

func assert_path_eq(path: felt**, expected_path: felt**, len: felt, index: felt) {
    if (index == len) {
        return ();
    }

    let path_ptr: felt* = cast([path + index], felt*);
    let expected_ptr: felt* = cast([expected_path + index], felt*);
    assert_bits32_eq(path_ptr, expected_ptr);

    return assert_path_eq(path, expected_path, len, index + 1);
}

func assert_uint384_array_eq(values: UInt384*, expected: UInt384*, len: felt, index: felt) {
    if (index == len) {
        return ();
    }

    let a = values[index];
    let b = expected[index];
    assert_uint384_eq(a, b);

    return assert_uint384_array_eq(values, expected, len, index + 1);
}
