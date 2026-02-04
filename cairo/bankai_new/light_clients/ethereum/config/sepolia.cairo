from starkware.cairo.common.registers import get_label_location
from starkware.cairo.common.uint256 import Uint256
from cairo.bankai_new.light_clients.ethereum.config.types import EthereumConfig, Hardforks

from cairo.bankai_new.debug.print import info_string, info_felt_hex

func get_config_sepolia() -> EthereumConfig {
    let config = EthereumConfig(
        network_id=1,
        sync_committee_period=8192,
        genesis_validator_root=0x19c80d8a5de5427e66d4cb600f6ac995061f73156996c9a7fca2285ae90169f,
    );
    return config;
}

func get_fork_data_sepolia{range_check_ptr}(fork_id: felt) -> (version: felt, slot: felt) {
    alloc_locals;

    assert [range_check_ptr] = Hardforks.N - 1 - fork_id;  // Check fork_id is valid (0-6)
    tempvar range_check_ptr = range_check_ptr + 1;

    let (schedule) = get_fork_schedule_sepolia();

    local version = [schedule + (fork_id * 2)];
    local slot = [schedule + (fork_id * 2) + 1];
    return (version, slot);
}

func get_fork_schedule_sepolia() -> (felt*) {
    return get_label_location(fork_schedule);

    fork_schedule:
    // SEPOLIA fork data (version, slot)
    // GENESIS
    dw 0x90000069000000000000000000000000;  // GENESIS_FORK_VERSION
    dw 0;  // GENESIS_ACTIVATION_SLOT
    // ALTAIR
    dw 0x90000070000000000000000000000000;  // ALTAIR_FORK_VERSION
    dw 1600;  // ALTAIR_ACTIVATION_SLOT (50 * 32)
    // BELLATRIX
    dw 0x90000071000000000000000000000000;  // BELLATRIX_FORK_VERSION
    dw 3200;  // BELLATRIX_ACTIVATION_SLOT (100 * 32)
    // CAPELLA
    dw 0x90000072000000000000000000000000;  // CAPELLA_FORK_VERSION
    dw 1818624;  // CAPELLA_ACTIVATION_SLOT (56832 * 32)
    // DENEB
    dw 0x90000073000000000000000000000000;  // DENEB_FORK_VERSION
    dw 4243456;  // DENEB_ACTIVATION_SLOT (132608 * 32)
    // ELECTRA
    dw 0x90000074000000000000000000000000;  // ELECTRA_FORK_VERSION
    dw 7118848;  // ELECTRA_ACTIVATION_SLOT (222464 * 32)
    // FULU
    dw 0x90000075000000000000000000000000;  // FULU_FORK_VERSION
    dw 8724480;  // FULU_ACTIVATION_SLOT (272640 * 32)
}

func get_fork_domain_sepolia(fork: felt) -> Uint256 {
    alloc_locals;

    let (data_address) = get_label_location(domain_data_sepolia);
    local low = [data_address + (fork * 2)];
    local high = [data_address + (fork * 2 + 1)];
    return (Uint256(low=low, high=high));

    domain_data_sepolia:
    dw 0x5f699a49ccd9b3fd666c35d4ae5f79e;  // Genesis low
    dw 0x7000000a8fee8ee9978418b64f1140b;  // Genesis high

    dw 0x32399a96f89d5ce37f1b875852afd540;  // Altair low
    dw 0x70000002944546c0d50cbdfd9448dfc;  // Altair high

    dw 0x10839ea6dcaaaa6372e95478610d7e08;  // Bellatrix low
    dw 0x700000036fa50131482fe2af396daf2;  // Bellatrix high

    dw 0x60f0a2ed78c1a85f0654941a0d19d0fa;  // Capella low
    dw 0x700000047eb72b3be36f08feffcaba7;  // Capella high

    dw 0x55fcf34b7e308f8fbca8e663bf565808;  // Deneb low
    dw 0x7000000d31f6191ca65c836e170318c;  // Deneb high

    dw 0x5b64eb2f9c81e0683f21dd0491e95aaa;  // Electra low
    dw 0x700000014045b5a1d8da091c2ee9e63;  // Electra high

    dw 0x22af469210b5b2c8807e372b6b9ca539;  // Fulu low
    dw 0x7000000f52c15272cff99835cd05aa5;  // Fulu high
}