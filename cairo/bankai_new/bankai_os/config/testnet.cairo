from cairo.bankai_new.bankai_os.config.types import BankaiOSConfig, Networks

func get_testnet_config() -> (config: BankaiOSConfig) {
    let config = BankaiOSConfig(
        version=1,
        network_id=Networks.TESTNET,
    );
    return (config=config);
}