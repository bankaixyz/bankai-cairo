use crate::error::Error;

pub struct ApiClient {
    pub base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn fetch_proof_by_beacon_height(
        &self,
        height: u64,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/beacon/{}", self.base_url, height);
        let response = reqwest::get(url)
            .await
            .map_err(|e| Error::ProofParsingError(e.to_string()))?;
        let proof: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::ProofParsingError(e.to_string()))?;
        Ok(proof)
    }

    pub async fn fetch_proof_by_execution_height(
        &self,
        height: u64,
    ) -> Result<serde_json::Value, Error> {
        let url = format!("{}/execution/{}", self.base_url, height);
        let response = reqwest::get(url)
            .await
            .map_err(|e| Error::ProofParsingError(e.to_string()))?;
        let proof: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::ProofParsingError(e.to_string()))?;
        Ok(proof)
    }
}
