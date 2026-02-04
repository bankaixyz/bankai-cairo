from starkware.cairo.common.cairo_builtins import KeccakBuiltin, BitwiseBuiltin
from starkware.cairo.common.registers import get_label_location
from starkware.cairo.common.uint256 import Uint256

from cairo.src.debug.print import info_string, info_felt_hex

from cairo.src.light_clients.ethereum.utils.ssz import SSZ
from cairo.src.utils.utils import felt_divmod

from cairo.src.light_clients.ethereum.config.config import (
    get_fork_schedule,
    get_genesis_validator_root,
    get_fork_data,
    get_fork_domain,
)
from cairo.src.light_clients.ethereum.config.types import Hardforks, Networks

namespace Fork {
    func get_root{
        range_check_ptr, bitwise_ptr: BitwiseBuiltin*, pow2_array: felt*, sha256_ptr: felt*
    }(network_id: felt, slot: felt) -> Uint256 {
        alloc_locals;
        let fork_id = Fork.get_id(network_id, slot);
        let genesis_validator_root = get_genesis_validator_root(network_id);

        let root = SSZ.hash_pair_container(Uint256(low=0, high=fork_id), genesis_validator_root);
        return root;
    }

    func get_id{range_check_ptr}(network_id: felt, slot: felt) -> (felt, felt) {
        alloc_locals;

        local fork: felt;
        // We load the network's fork schedule into memory
        // We can then use the loaded data within the hint to derive the correct fork id
        // The enables us to reuse the same schedule between rust and cairo
        let (fork_schedule) = get_fork_schedule(network_id);
        let n_hardforks = Hardforks.N;
        %{ check_fork_version() %}

        // We now validate the hint using range checks
        if (fork == Hardforks.GENESIS) {
            let (fork_id, _) = get_fork_data(network_id, Hardforks.GENESIS);
            let (_, altair_activation_slot) = get_fork_data(network_id, Hardforks.ALTAIR);
            assert [range_check_ptr] = altair_activation_slot - slot;
            tempvar range_check_ptr = range_check_ptr + 1;
            return (fork_id, fork);
        }

        if (fork == Hardforks.ALTAIR) {
            let (fork_id, altair_activation_slot) = get_fork_data(network_id, Hardforks.ALTAIR);
            let (_, bellatrix_activation_slot) = get_fork_data(network_id, Hardforks.BELLATRIX);
            assert [range_check_ptr] = bellatrix_activation_slot - slot;
            assert [range_check_ptr + 1] = slot - altair_activation_slot;
            tempvar range_check_ptr = range_check_ptr + 2;
            return (fork_id, fork);
        }

        if (fork == Hardforks.BELLATRIX) {
            let (fork_id, bellatrix_activation_slot) = get_fork_data(
                network_id, Hardforks.BELLATRIX
            );
            let (_, capella_activation_slot) = get_fork_data(network_id, Hardforks.CAPELLA);
            assert [range_check_ptr] = capella_activation_slot - slot;
            assert [range_check_ptr + 1] = slot - bellatrix_activation_slot;
            tempvar range_check_ptr = range_check_ptr + 2;
            return (fork_id, fork);
        }

        if (fork == Hardforks.CAPELLA) {
            let (fork_id, capella_activation_slot) = get_fork_data(network_id, Hardforks.CAPELLA);
            let (_, deneb_activation_slot) = get_fork_data(network_id, Hardforks.DENEB);
            assert [range_check_ptr] = deneb_activation_slot - slot;
            assert [range_check_ptr + 1] = slot - capella_activation_slot;
            tempvar range_check_ptr = range_check_ptr + 2;
            return (fork_id, fork);
        }

        if (fork == Hardforks.DENEB) {
            let (fork_id, deneb_activation_slot) = get_fork_data(network_id, Hardforks.DENEB);
            let (_, electra_activation_slot) = get_fork_data(network_id, Hardforks.ELECTRA);
            assert [range_check_ptr] = electra_activation_slot - slot;
            assert [range_check_ptr + 1] = slot - deneb_activation_slot;
            tempvar range_check_ptr = range_check_ptr + 2;
            return (fork_id, fork);
        }

        if (fork == Hardforks.ELECTRA) {
            let (fork_id, electra_activation_slot) = get_fork_data(network_id, Hardforks.ELECTRA);
            let (_, fulu_activation_slot) = get_fork_data(network_id, Hardforks.FULU);
            assert [range_check_ptr] = fulu_activation_slot - slot;
            assert [range_check_ptr + 1] = slot - electra_activation_slot;
            tempvar range_check_ptr = range_check_ptr + 2;
            return (fork_id, fork);
        }

        if (fork == Hardforks.FULU) {
            let (fork_id, fulu_activation_slot) = get_fork_data(network_id, Hardforks.FULU);
            assert [range_check_ptr] = slot - fulu_activation_slot;
            tempvar range_check_ptr = range_check_ptr + 1;
            return (fork_id, fork);
        }

        assert 1 = 0;
        return (0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF);
    }
}

namespace Domain {
    const DOMAIN_SYNC_COMMITTEE = 0x07000000000000000000000000000000;

    func compute_signing_root{
        range_check_ptr, bitwise_ptr: BitwiseBuiltin*, pow2_array: felt*, sha256_ptr: felt*
    }(network_id: felt, message: Uint256, slot: felt) -> Uint256 {
        let domain = get_domain(network_id, slot);
        let root = SSZ.hash_pair_container(message, domain);
        return root;
    }

    func get_domain{range_check_ptr}(network_id: felt, slot: felt) -> Uint256 {
        alloc_locals;

        let (fork_id, fork) = Fork.get_id(network_id, slot);
        let fork_domain = get_fork_domain(network_id, fork);
        return fork_domain;
    }

    func compute{
        range_check_ptr, bitwise_ptr: BitwiseBuiltin*, pow2_array: felt*, sha256_ptr: felt*
    }(network_id: felt, slot: felt) -> Uint256 {
        let fork_root = Fork.get_root(network_id, slot);

        // We now need to right right-shift the fork root 4 bytes, and prepend the domain
        let (q_high, r_high) = felt_divmod(fork_root.high, 0x100000000);
        let (q_low, _r_low) = felt_divmod(fork_root.low, 0x100000000);

        let high = DOMAIN_SYNC_COMMITTEE + q_high;
        let low = r_high * 0x1000000000000000000000000 + q_low;

        return (Uint256(low=low, high=high));
    }
}
