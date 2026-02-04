%builtins output pedersen range_check ecdsa bitwise ec_op keccak poseidon range_check96 add_mod mul_mod
from starkware.cairo.common.cairo_builtins import (
    BitwiseBuiltin,
    KeccakBuiltin,
    PoseidonBuiltin,
    HashBuiltin,
    ModBuiltin,
)
from starkware.cairo.common.cairo_keccak.keccak import finalize_keccak
from starkware.cairo.common.alloc import alloc
from sha import SHA256

from cairo.src.bankai_os.lib import run_bankai_os
from cairo.src.utils.utils import pow2alloc128

func main{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    ecdsa_ptr: felt*,
    bitwise_ptr: BitwiseBuiltin*,
    ec_op_ptr: felt*,
    keccak_ptr: felt*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
}() {
    alloc_locals;

    let (keccak_felt_ptr: felt*) = alloc();
    let start_keccak_felt_ptr = keccak_felt_ptr;
    let (sha256_ptr, sha256_ptr_start) = SHA256.init();
    let (pow2_array) = pow2alloc128();

    run_bankai_os{
        output_ptr=output_ptr,
        pedersen_ptr=pedersen_ptr,
        range_check_ptr=range_check_ptr,
        bitwise_ptr=bitwise_ptr,
        keccak_ptr=keccak_felt_ptr,
        poseidon_ptr=poseidon_ptr,
        range_check96_ptr=range_check96_ptr,
        add_mod_ptr=add_mod_ptr,
        mul_mod_ptr=mul_mod_ptr,
        sha256_ptr=sha256_ptr,
        pow2_array=pow2_array,
    }();

    SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);
    finalize_keccak(keccak_ptr_start=start_keccak_felt_ptr, keccak_ptr_end=keccak_felt_ptr);

    return ();
}
