from cairo.src.bankai_os.config.testnet import get_testnet_config
from cairo.src.bankai_os.config.types import BankaiOSConfig, Networks

func get_config(network_id: felt) -> (config: BankaiOSConfig) {
    if (network_id == Networks.MAINNET) {
        // ToDo: Add mainnet config
        assert 1 = 0;

        let (config) = get_testnet_config();
        return (config=config);
    } else {
        let (config) = get_testnet_config();
        return (config=config);
    }
}