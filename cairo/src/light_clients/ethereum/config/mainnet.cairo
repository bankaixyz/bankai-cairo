from starkware.cairo.common.registers import get_label_location
from starkware.cairo.common.uint256 import Uint256

from cairo.src.light_clients.ethereum.config.types import EthereumConfig, Hardforks

func get_config_mainnet() -> EthereumConfig {
    let config = EthereumConfig(
        network_id=0,
        sync_committee_period=8192,
        genesis_validator_root=0x02e93bd4b4aff8bfc81c9f5f468e74c6c86a8dc4d368bcfe9874bc8742e06880,
    );
    return config;
}

func get_fork_data_mainnet{range_check_ptr}(fork_id: felt) -> (version: felt, slot: felt) {
    alloc_locals;

    assert [range_check_ptr] = Hardforks.N - 1 - fork_id;  // Check fork_id is valid (0-6)
    tempvar range_check_ptr = range_check_ptr + 1;

    let (schedule) = get_fork_schedule_mainnet();

    local version = [schedule + (fork_id * 2)];
    local slot = [schedule + (fork_id * 2) + 1];
    return (version, slot);
}

func get_fork_schedule_mainnet() -> (felt*) {
    return get_label_location(fork_schedule);

    fork_schedule:
    // MAINNET fork data (version, slot)
    // GENESIS
    dw 0x00000000000000000000000000000000;  // GENESIS_FORK_VERSION
    dw 0;  // GENESIS_ACTIVATION_SLOT
    // ALTAIR
    dw 0x01000000000000000000000000000000;  // ALTAIR_FORK_VERSION
    dw 2375680;  // ALTAIR_ACTIVATION_SLOT (74240 * 32)
    // BELLATRIX
    dw 0x02000000000000000000000000000000;  // BELLATRIX_FORK_VERSION
    dw 4636672;  // BELLATRIX_ACTIVATION_SLOT (144896 * 32)
    // CAPELLA
    dw 0x03000000000000000000000000000000;  // CAPELLA_FORK_VERSION
    dw 6209536;  // CAPELLA_ACTIVATION_SLOT (194048 * 32)
    // DENEB
    dw 0x04000000000000000000000000000000;  // DENEB_FORK_VERSION
    dw 8626176;  // DENEB_ACTIVATION_SLOT (269568 * 32)
    // ELECTRA
    dw 0x05000000000000000000000000000000;  // ELECTRA_FORK_VERSION
    dw 11649024;  // ELECTRA_ACTIVATION_SLOT (222464 * 32)
    // FULU
    dw 0x06000000000000000000000000000000;  // FULU_FORK_VERSION
    dw 13164544;  // FULU_ACTIVATION_SLOT (411392 * 32)
}

func get_fork_domain_mainnet(fork_id: felt) -> Uint256 {
    alloc_locals;

    let (data_address) = get_label_location(domain_data_mainnet);
    let low = [data_address + fork_id * 2];
    let high = [data_address + fork_id * 2 + 1];
    return (Uint256(low=low, high=high));

    // MAINNET dummy precomputed domain values.
    domain_data_mainnet:
    dw 0x2350947421a3e4a979779642cfdb0f66;  // Genesis low
    dw 0x7000000b5303f2ad2010d699a76c8e6;  // Genesis high

    dw 0x31016c31b4da651f362045e02b4447f0;  // Altair low
    dw 0x7000000c3442b13b42f0f3c37034be9;  // Altair high

    dw 0x40848881a8d4f0af0be83417a85c0f45;  // Bellatrix low
    dw 0x70000004a26c58b08add8089b75caa5;  // Bellatrix high

    dw 0x69bf583a7f9e0af049305b62de676640;  // Capella low
    dw 0x7000000bba4da96354c9f25476cf1bc;  // Capella high

    dw 0x883b712607f952d5198d0f5677564636;  // Deneb low
    dw 0x70000006a95a1a967855d676d48be69;  // Deneb high

    dw 0x883b712607f952d5198d0f5677564636;  // Electra low
    dw 0x70000006a95a1a967855d676d48be69;  // Electra high

    dw 0x7ac5f562cf682ce6bc41b8ec28ba1a07;  // Fulu low
    dw 0x700000082fae541f8a3db43adb5e799;  // Fulu high
}