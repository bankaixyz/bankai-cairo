struct EthereumConfig {
    network_id: felt,
    sync_committee_period: felt,
    genesis_validator_root: felt,
}

namespace Networks {
    const MAINNET = 0;
    const SEPOLIA = 1;
}

namespace Hardforks {
    // Fork IDs
    const GENESIS = 0;
    const ALTAIR = 1;
    const BELLATRIX = 2;
    const CAPELLA = 3;
    const DENEB = 4;
    const ELECTRA = 5;
    const FULU = 6;

    const N = 7;
}