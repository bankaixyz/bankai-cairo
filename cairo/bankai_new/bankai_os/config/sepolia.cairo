from cairo.bankai_new.bankai_os.config.types import BankaiOSConfig

func get_sepolia_config() -> (config: BankaiOSConfig) {
    let config = BankaiOSConfig(
        version=1,
        network_id=Networks.SEPOLIA,
    );
    return (config=config);
}