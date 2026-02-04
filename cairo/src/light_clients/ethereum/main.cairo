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
from cairo.src.utils.utils import felt_divmod, pow2alloc128

from cairo.src.light_clients.ethereum.lib import run
from cairo.src.light_clients.ethereum.types import get_ethereum_genesis
from cairo.src.light_clients.ethereum.config.config import get_config, Networks

func main{
    output_ptr: felt*,
    pedersen_ptr: felt*,
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

    let network_id = Networks.SEPOLIA;
    let config = get_config(network_id=network_id);

    let (block) = get_ethereum_genesis(config.genesis_validator_root);

    let (pow2_array) = pow2alloc128();

    run{
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
    }(block, network_id);

    SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);
    finalize_keccak(keccak_ptr_start=start_keccak_felt_ptr, keccak_ptr_end=keccak_felt_ptr);

    return ();
}
