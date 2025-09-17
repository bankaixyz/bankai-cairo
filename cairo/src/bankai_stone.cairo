%builtins output pedersen range_check bitwise keccak poseidon range_check96 add_mod mul_mod

from starkware.cairo.common.cairo_builtins import (
    BitwiseBuiltin,
    KeccakBuiltin,
    PoseidonBuiltin,
    HashBuiltin,
    ModBuiltin,
)
from starkware.cairo.common.cairo_keccak.keccak import finalize_keccak
from starkware.cairo.common.alloc import alloc

from cairo.src.lib import run_bankai

const BOOTLOADER_PROGRAM_HASH = 0x5AB580B04E3532B6B18F81CFA654A05E29DD8E2352D88DF1E765A84072DB07;
const SYNC_COMMITTEE_PERIOD = 8192;
const USE_BUILTIN_KECCAK = 0;

func main{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    keccak_ptr: KeccakBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
}() {
    alloc_locals;

    if (USE_BUILTIN_KECCAK == 1) {

        let keccak_felt_ptr = cast(keccak_ptr, felt*);
        run_bankai{
            output_ptr=output_ptr,
            pedersen_ptr=pedersen_ptr,
            range_check_ptr=range_check_ptr,
            bitwise_ptr=bitwise_ptr,
            keccak_ptr=keccak_felt_ptr,
            poseidon_ptr=poseidon_ptr,
            range_check96_ptr=range_check96_ptr,
            add_mod_ptr=add_mod_ptr,
            mul_mod_ptr=mul_mod_ptr,
        }();
        tempvar keccak_ptr = cast(keccak_felt_ptr, KeccakBuiltin*);
        return ();
    } 

    let (keccak_felt_ptr: felt*) = alloc();
    let start_keccak_felt_ptr = keccak_felt_ptr;

    run_bankai{
        output_ptr=output_ptr,
        pedersen_ptr=pedersen_ptr,
        range_check_ptr=range_check_ptr,
        bitwise_ptr=bitwise_ptr,
        keccak_ptr=keccak_felt_ptr,
        poseidon_ptr=poseidon_ptr,
        range_check96_ptr=range_check96_ptr,
        add_mod_ptr=add_mod_ptr,
        mul_mod_ptr=mul_mod_ptr,
    }();

    finalize_keccak(keccak_ptr_start=start_keccak_felt_ptr, keccak_ptr_end=keccak_felt_ptr);

    return ();
}