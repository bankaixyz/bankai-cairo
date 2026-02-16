use crate::types::JobStage;
use std::convert::Infallible;
use thiserror::Error;
use tracing::warn;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

#[derive(Debug, Error)]
pub enum ProverErrorCode {
    #[error("PROVER_DECODING_FAILED")]
    DecodingFailed,
    #[error("PROVER_TRACE_GEN_FAILED")]
    TraceGenFailed,
    #[error("PROVER_PROVING_FAILED")]
    ProvingFailed,
    #[error("PROVER_PERSIST_FAILED")]
    PersistFailed,
    #[error("PROVER_STATE_CORRUPT")]
    StateCorrupt,
}

#[derive(Debug, Error)]
#[error("{code}: {message}")]
pub struct ProverError {
    pub stage: JobStage,
    pub code: ProverErrorCode,
    pub message: String,
}

#[derive(Debug, Error)]
#[error("{error_code}: {message}")]
pub struct HttpError {
    pub status: StatusCode,
    pub error_code: &'static str,
    pub message: String,
}

impl warp::reject::Reject for HttpError {}

pub fn bad_request(message: impl Into<String>) -> Rejection {
    warp::reject::custom(HttpError {
        status: StatusCode::BAD_REQUEST,
        error_code: "BAD_REQUEST",
        message: message.into(),
    })
}

pub fn unauthorized(message: impl Into<String>) -> Rejection {
    warp::reject::custom(HttpError {
        status: StatusCode::UNAUTHORIZED,
        error_code: "UNAUTHORIZED",
        message: message.into(),
    })
}

pub fn conflict(message: impl Into<String>) -> Rejection {
    warp::reject::custom(HttpError {
        status: StatusCode::CONFLICT,
        error_code: "CONFLICT",
        message: message.into(),
    })
}

pub fn busy(message: impl Into<String>) -> Rejection {
    warp::reject::custom(HttpError {
        status: StatusCode::SERVICE_UNAVAILABLE,
        error_code: "BUSY",
        message: message.into(),
    })
}

pub fn not_found(message: impl Into<String>) -> Rejection {
    warp::reject::custom(HttpError {
        status: StatusCode::NOT_FOUND,
        error_code: "NOT_FOUND",
        message: message.into(),
    })
}

pub fn internal(message: impl Into<String>) -> Rejection {
    warp::reject::custom(HttpError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        error_code: "INTERNAL_SERVER_ERROR",
        message: message.into(),
    })
}

pub async fn recover(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(http) = err.find::<HttpError>() {
        let reply =
            crate::http::replies::json_error(http.status, http.error_code, http.message.clone());
        return Ok(reply);
    }

    if err.is_not_found() {
        return Ok(crate::http::replies::json_error(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "route not found",
        ));
    }

    // Warp's JSON body parse failures arrive as a rejection; keep the message reasonably descriptive.
    if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        return Ok(crate::http::replies::json_error(
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
            format!("invalid json body: {e}"),
        ));
    }

    warn!(?err, "unhandled rejection");
    Ok(crate::http::replies::json_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_SERVER_ERROR",
        "internal server error",
    ))
}
