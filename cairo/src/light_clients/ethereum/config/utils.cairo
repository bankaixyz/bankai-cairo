// from debug import print_string, print_felt_hex

// from cairo.src.light_clients.ethereum.utils.domain import Domain
// from cairo.src.light_clients.ethereum.config.config import get_fork_data
// from cairo.src.light_clients.ethereum.config.types import Networks

// // this function is used to compute the domain values found in get_domain()
// func precompute_domains{
//     range_check_ptr, bitwise_ptr: BitwiseBuiltin*, pow2_array: felt*, sha256_ptr: felt*
// }() {
//     alloc_locals;

// // Precompute domains for MAINNET.
//     let (_, genesis_slot_mainnet) = Network.get_fork_data(Network.MAINNET, Network.GENESIS);
//     let domain_mainnet_genesis = Domain.compute(Network.MAINNET, genesis_slot_mainnet);
//     print_string('domain_mainnet_genesis');
//     print_uint256(domain_mainnet_genesis);

// let (_, altair_slot_mainnet) = Network.get_fork_data(Network.MAINNET, Network.ALTAIR);
//     let domain_mainnet_altair = Domain.compute(Network.MAINNET, altair_slot_mainnet);
//     print_string('domain_mainnet_altair');
//     print_uint256(domain_mainnet_altair);

// let (_, bellatrix_slot_mainnet) = Network.get_fork_data(Network.MAINNET, Network.BELLATRIX);
//     let domain_mainnet_bellatrix = Domain.compute(Network.MAINNET, bellatrix_slot_mainnet);
//     print_string('domain_mainnet_bellatrix');
//     print_uint256(domain_mainnet_bellatrix);

// let (_, capella_slot_mainnet) = Network.get_fork_data(Network.MAINNET, Network.CAPELLA);
//     let domain_mainnet_capella = Domain.compute(Network.MAINNET, capella_slot_mainnet);
//     print_string('domain_mainnet_capella');
//     print_uint256(domain_mainnet_capella);

// let (_, deneb_slot_mainnet) = Network.get_fork_data(Network.MAINNET, Network.DENEB);
//     let domain_mainnet_deneb = Domain.compute(Network.MAINNET, deneb_slot_mainnet);
//     print_string('domain_mainnet_deneb');
//     print_uint256(domain_mainnet_deneb);

// let (_, electra_slot_mainnet) = Network.get_fork_data(Network.MAINNET, Network.ELECTRA);
//     print_string('got fork data');
//     let domain_mainnet_electra = Domain.compute(Network.MAINNET, electra_slot_mainnet - 1);
//     print_string('domain_mainnet_electra');
//     print_uint256(domain_mainnet_electra);

// let (_, fulu_slot_mainnet) = Network.get_fork_data(Network.MAINNET, Network.FULU);
//     let domain_mainnet_fulu = Domain.compute(Network.MAINNET, fulu_slot_mainnet);
//     print_string('domain_mainnet_fulu');
//     print_uint256(domain_mainnet_fulu);

// // Precompute domains for SEPOLIA.
//     let (_, genesis_slot_sepolia) = Network.get_fork_data(Network.SEPOLIA, Network.GENESIS);
//     let domain_sepolia_genesis = Domain.compute(Network.SEPOLIA, genesis_slot_sepolia);
//     print_string('domain_sepolia_genesis');
//     print_uint256(domain_sepolia_genesis);

// let (_, altair_slot_sepolia) = Network.get_fork_data(Network.SEPOLIA, Network.ALTAIR);
//     let domain_sepolia_altair = Domain.compute(Network.SEPOLIA, altair_slot_sepolia);
//     print_string('domain_sepolia_altair');
//     print_uint256(domain_sepolia_altair);

// let (_, bellatrix_slot_sepolia) = Network.get_fork_data(Network.SEPOLIA, Network.BELLATRIX);
//     let domain_sepolia_bellatrix = Domain.compute(Network.SEPOLIA, bellatrix_slot_sepolia);
//     print_string('domain_sepolia_bellatrix');
//     print_uint256(domain_sepolia_bellatrix);

// let (_, capella_slot_sepolia) = Network.get_fork_data(Network.SEPOLIA, Network.CAPELLA);
//     let domain_sepolia_capella = Domain.compute(Network.SEPOLIA, capella_slot_sepolia);
//     print_string('domain_sepolia_capella');
//     print_uint256(domain_sepolia_capella);

// let (_, deneb_slot_sepolia) = Network.get_fork_data(Network.SEPOLIA, Network.DENEB);
//     let domain_sepolia_deneb = Domain.compute(Network.SEPOLIA, deneb_slot_sepolia);
//     print_string('domain_sepolia_deneb');
//     print_uint256(domain_sepolia_deneb);

// let (_, electra_slot_sepolia) = Network.get_fork_data(Network.SEPOLIA, Network.ELECTRA);
//     let domain_sepolia_electra = Domain.compute(Network.SEPOLIA, electra_slot_sepolia);
//     print_string('domain_sepolia_electra');
//     print_uint256(domain_sepolia_electra);

// let (_, fulu_slot_sepolia) = Network.get_fork_data(Network.SEPOLIA, Network.FULU);
//     print_string('giot slot');
//     let domain_sepolia_fulu = Domain.compute(Network.SEPOLIA, fulu_slot_sepolia);
//     print_string('domain_sepolia_fulu');
//     print_uint256(domain_sepolia_fulu);

// return ();
// }

// func print_uint256(value: Uint256) {
//     print_string('Uint256:');
//     print_felt_hex(value.low);
//     print_felt_hex(value.high);

// return ();
// }
