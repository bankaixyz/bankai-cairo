%builtins output pedersen range_check ecdsa bitwise ec_op keccak poseidon range_check96 add_mod mul_mod

from starkware.cairo.common.cairo_builtins import PoseidonBuiltin, ModBuiltin, BitwiseBuiltin, HashBuiltin
from cairo.src.verify_stone import verify_cairo_proof

from starkware.cairo.common.registers import get_fp_and_pc
from cairo.src.utils import pow2alloc128
from sha import SHA256
from cairo.src.types import EpochUpdate
from cairo.src.verify_epoch import run_epoch_update

func main{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
}() {
    alloc_locals;

    let (pow2_array) = pow2alloc128();
    let (sha256_ptr, sha256_ptr_start) = SHA256.init();

    // local is_genesis: felt;
    // local update_committee: felt;
    // %{ set_program_route() %}

    local expected_committee_hash: Uint256;
    if (is_genesis == 1) {
        %{ write_genesis_inputs() %}
    } else {
        %{ write_update_inputs() %}
    }

    local epoch_update: EpochUpdate;
    %{ write_epoch_update_inputs() %}

    with pow2_array, sha256_ptr {
        run_epoch_update(epoch_update); 
    }

    %{ verify_epoch_update_outputs() %}

    SHA256.finalize(sha256_start_ptr=sha256_ptr_start, sha256_end_ptr=sha256_ptr);

    let (proof_program_hash, output_hash) = verify_cairo_proof();

    return ();
}


