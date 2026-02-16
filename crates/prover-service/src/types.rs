use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitProofRequest {
    pub request_id: String,
    pub idempotency_key: String,
    pub payload_type: String,
    pub payload_json: Value,
    pub metadata: SubmitMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitMetadata {
    pub producer_id: String,
    pub producer_attempt: Option<u32>,
    pub max_primary_attempts: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubmitProofResponse {
    pub request_id: String,
    pub status: JobStatus,
    pub stage: JobStage,
    pub status_url: String,
    pub proof_url: String,
    pub ws_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusResponse {
    pub request_id: String,
    pub status: JobStatus,
    pub stage: JobStage,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStage {
    Queued,
    Decoding,
    TraceGen,
    Proving,
    Persisting,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub request_id: String,
    pub idempotency_key: String,
    pub status: JobStatus,
    pub stage: JobStage,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub end_to_end_ms: Option<u64>,
    pub trace_gen_ms: Option<u64>,
    pub proving_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientWsMessage {
    Subscribe { request_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerWsMessage {
    Status {
        request_id: String,
        status: JobStatus,
        stage: JobStage,
        error_code: Option<String>,
        error_message: Option<String>,
    },
}

impl ServerWsMessage {
    pub fn status_from_job(job: &JobRecord) -> Self {
        ServerWsMessage::Status {
            request_id: job.request_id.clone(),
            status: job.status.clone(),
            stage: job.stage,
            error_code: job.error_code.clone(),
            error_message: job.error_message.clone(),
        }
    }
}

pub const SUPPORTED_PAYLOAD_TYPE: &str = "bankai_block_bundle_cairo";
