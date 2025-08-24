#[derive(Clone, Debug)]
pub struct BankaiConfig {
    pub atlantic_endpoint: String,
    pub database_url: String,
}

impl Default for BankaiConfig {
    fn default() -> Self {
        Self {
            atlantic_endpoint: "https://staging.atlantic.api.herodotus.cloud".to_string(),
            database_url: "sqlite:./sqlite_state/bankai.db".to_string(),
        }
    }
}

impl BankaiConfig {
    pub fn docker_config() -> Self {
        Self {
            atlantic_endpoint: "https://staging.atlantic.api.herodotus.cloud".to_string(),
            database_url: "sqlite:./sqlite_state/bankai.db".to_string(),
        }
    }
}
