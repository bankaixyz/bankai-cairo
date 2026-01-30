from starkware.cairo.common.uint256 import Uint256

from cairo.bankai_new.light_clients.ethereum.config.mainnet import get_config_mainnet, get_fork_data_mainnet, get_fork_domain_mainnet, get_fork_schedule_mainnet
from cairo.bankai_new.light_clients.ethereum.config.sepolia import get_config_sepolia, get_fork_data_sepolia, get_fork_domain_sepolia, get_fork_schedule_sepolia
from cairo.bankai_new.light_clients.ethereum.config.types import EthereumConfig, Networks

func get_config(network_id: felt) -> EthereumConfig {
    if (network_id == Networks.MAINNET) {
        let config = get_config_mainnet();
        return config;
    } else {
        let config = get_config_sepolia();
        return config;
    }
}

// Data structure for fork versions and activation slots
func get_fork_data{range_check_ptr}(network_id: felt, fork_id: felt) -> (
    version: felt, slot: felt
) {
    if (network_id == Networks.MAINNET) {
        return get_fork_data_mainnet(fork_id);
    } else {
        return get_fork_data_sepolia(fork_id);
    }
}

// Get the domain for a given fork
func get_fork_domain(network_id: felt, fork_id: felt) -> Uint256 {
    if (network_id == Networks.MAINNET) {
        return get_fork_domain_mainnet(fork_id);
    } else {
        return get_fork_domain_sepolia(fork_id);
    }
}

func get_genesis_validator_root(network_id: felt) -> Uint256 {
    if (network_id == Networks.MAINNET) {
        return (
            Uint256(
                low=0x54bfe9f06bf33ff6cf5ad27f511bfe95, high=0x4b363db94e286120d76eb905340fdd4e
            )
        );
    } else {
        return (
            Uint256(
                low=0xcf3f9209c00e4efbaaddac09ed9b8078, high=0xd8ea171f3c94aea21ebc42a1ed61052a
            )
        );
    }
}

func get_fork_schedule(network_id: felt) -> (felt*) {
    if (network_id == Networks.MAINNET) {
        return get_fork_schedule_mainnet();
    } else {
        return get_fork_schedule_sepolia();
    }
}