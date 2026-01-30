from cairo.bankai_new.bankai_os.config.sepolia import get_sepolia_config
from cairo.bankai_new.bankai_os.config.types import BankaiOSConfig

namespace Networks {
    const MAINNET = 0;
    const TESTNET = 1;
}

func get_config{network_id: felt}() -> (config: BankaiOSConfig) {
    if (network_id == EthereumNetwork.MAINNET) {
        // ToDo: Add mainnet config
        assert 1 = 0;

        return (config=get_sepolia_config());
    } else {
        return (config=get_sepolia_config());
    }
}