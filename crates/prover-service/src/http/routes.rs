use crate::app::AppState;
use crate::errors;
use crate::types::{
    JobRecord, JobStatus, StatusResponse, SubmitProofRequest, SubmitProofResponse,
    SUPPORTED_PAYLOAD_TYPE,
};
use crate::ws;
use serde_json::Value;
use std::convert::Infallible;
use tokio::sync::OwnedSemaphorePermit;
use tracing::{info, instrument, warn};
use warp::http::StatusCode;
use warp::{Filter, Reply};

pub fn routes(state: AppState) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone
{
    let v1 = warp::path("v1");

    let submit = v1
        .and(warp::path("proofs"))
        .and(warp::post())
        .and(with_state(state.clone()))
        .and(auth(state.clone()))
        .and(warp::body::json())
        .and_then(handle_submit);

    let status = v1
        .and(warp::path("proofs"))
        .and(warp::path::param::<String>())
        .and(warp::path("status"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and(auth(state.clone()))
        .and_then(handle_status);

    let proof = v1
        .and(warp::path("proofs"))
        .and(warp::path::param::<String>())
        .and(warp::path("proof"))
        .and(warp::get())
        .and(with_state(state.clone()))
        .and(auth(state.clone()))
        .and_then(handle_proof);

    let ws_state = state.clone();
    let ws_route = v1
        .and(warp::path("proofs"))
        .and(warp::path("ws"))
        .and(warp::ws())
        .and(with_state(ws_state.clone()))
        .and(auth_ws(ws_state))
        .map(|ws: warp::ws::Ws, state: AppState| ws.on_upgrade(move |socket| ws::handle_ws(socket, state)));

    submit.or(status).or(proof).or(ws_route)
}

fn with_state(
    state: AppState,
) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn auth(state: AppState) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("authorization").and_then(move |auth: Option<String>| {
        let state = state.clone();
        async move {
            if let Some(expected) = &state.config.auth_token {
                let provided = auth.ok_or_else(|| errors::unauthorized("missing authorization header"))?;
                let provided = provided
                    .strip_prefix("Bearer ")
                    .ok_or_else(|| errors::unauthorized("expected 'Authorization: Bearer <token>'"))?;
                if provided != expected {
                    return Err(errors::unauthorized("invalid bearer token"));
                }
            }
            Ok::<(), warp::Rejection>(())
        }
    }).untuple_one()
}

fn auth_ws(state: AppState) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    // Warp's ws() filter doesn't give us the request after upgrade, so we authenticate before upgrade.
    warp::header::optional::<String>("authorization").and_then(move |auth: Option<String>| {
        let state = state.clone();
        async move {
            if let Some(expected) = &state.config.auth_token {
                let provided = auth.ok_or_else(|| errors::unauthorized("missing authorization header"))?;
                let provided = provided
                    .strip_prefix("Bearer ")
                    .ok_or_else(|| errors::unauthorized("expected 'Authorization: Bearer <token>'"))?;
                if provided != expected {
                    return Err(errors::unauthorized("invalid bearer token"));
                }
            }
            Ok::<(), warp::Rejection>(())
        }
    }).untuple_one()
}

#[instrument(skip_all, fields(request_id = %req.request_id, idempotency_key = %req.idempotency_key))]
async fn handle_submit(state: AppState, req: SubmitProofRequest) -> Result<impl Reply, warp::Rejection> {
    validate_submit(&req)?;

    if let Some(job) = state
        .fs
        .read_job(&req.request_id)
        .await
        .map_err(|e| errors::internal(e.to_string()))?
    {
        return Ok(warp::reply::with_status(
            warp::reply::json(&submit_response(&job)),
            StatusCode::ACCEPTED,
        ));
    }

    let permit = state
        .semaphore
        .clone()
        .try_acquire_owned()
        .map_err(|_| errors::busy("prover is busy processing another request"))?;

    let now = crate::app::now_ms();
    let job = JobRecord {
        request_id: req.request_id.clone(),
        idempotency_key: req.idempotency_key.clone(),
        status: JobStatus::Queued,
        stage: crate::types::JobStage::Queued,
        error_code: None,
        error_message: None,
        created_at_ms: now,
        updated_at_ms: now,
        end_to_end_ms: None,
        trace_gen_ms: None,
        proving_ms: None,
    };

    match state.fs.persist_new_job(&job, &req).await {
        Ok(()) => {
            state.registry.ensure(&job.request_id).await;
            state.broadcast_job(&job).await;
            spawn_job(state.clone(), job.request_id.clone(), permit);
            info!("started job");

            Ok(warp::reply::with_status(
                warp::reply::json(&submit_response(&job)),
                StatusCode::ACCEPTED,
            ))
        }
        Err(crate::state::fs::FsError::AlreadyExists) => {
            drop(permit);
            let job = state
                .fs
                .read_job(&req.request_id)
                .await
                .map_err(|e| errors::internal(e.to_string()))?
                .ok_or_else(|| errors::not_found("job state missing for existing request_id"))?;

            Ok(warp::reply::with_status(
                warp::reply::json(&submit_response(&job)),
                StatusCode::ACCEPTED,
            ))
        }
        Err(e) => {
            drop(permit);
            Err(errors::internal(e.to_string()))
        }
    }
}

fn spawn_job(state: AppState, request_id: String, permit: OwnedSemaphorePermit) {
    tokio::spawn(async move {
        let _permit = permit;
        if let Err(e) = crate::proving::pipeline::run_job(state.clone(), request_id.clone()).await {
            warn!(request_id, "job failed in submit loop: {e}");
        }
    });
}

fn validate_submit(req: &SubmitProofRequest) -> Result<(), warp::Rejection> {
    if req.request_id.trim().is_empty() {
        return Err(errors::bad_request("request_id is required"));
    }
    if req.idempotency_key.trim().is_empty() {
        return Err(errors::bad_request("idempotency_key is required"));
    }
    if req.payload_type.trim().is_empty() {
        return Err(errors::bad_request("payload_type is required"));
    }
    if req.payload_type != SUPPORTED_PAYLOAD_TYPE {
        return Err(errors::bad_request(format!(
            "unsupported payload_type: {}",
            req.payload_type
        )));
    }
    // payload_json is required; serde ensures presence, but keep a sanity check
    if matches!(req.payload_json, Value::Null) {
        return Err(errors::bad_request("payload_json is required"));
    }
    if req.metadata.producer_id.trim().is_empty() {
        return Err(errors::bad_request("metadata.producer_id is required"));
    }
    Ok(())
}

fn submit_response(job: &JobRecord) -> SubmitProofResponse {
    SubmitProofResponse {
        request_id: job.request_id.clone(),
        status: job.status.clone(),
        stage: job.stage,
        status_url: format!("/v1/proofs/{}/status", job.request_id),
        proof_url: format!("/v1/proofs/{}/proof", job.request_id),
        ws_url: "/v1/proofs/ws".to_string(),
    }
}

#[instrument(skip_all, fields(request_id = %request_id))]
async fn handle_status(
    request_id: String,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let job = state
        .fs
        .read_job(&request_id)
        .await
        .map_err(|e| errors::internal(e.to_string()))?
        .ok_or_else(|| errors::not_found("unknown request_id"))?;

    let body = StatusResponse {
        request_id: job.request_id.clone(),
        status: job.status,
        stage: job.stage,
        error_code: job.error_code.clone(),
        error_message: job.error_message.clone(),
    };
    Ok(warp::reply::json(&body))
}

#[instrument(skip_all, fields(request_id = %request_id))]
async fn handle_proof(
    request_id: String,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let job = state
        .fs
        .read_job(&request_id)
        .await
        .map_err(|e| errors::internal(e.to_string()))?
        .ok_or_else(|| errors::not_found("unknown request_id"))?;

    if job.status != JobStatus::Succeeded {
        return Err(errors::conflict(format!(
            "proof not ready; current status is {:?}",
            job.status
        )));
    }

    let proof_path = state
        .fs
        .find_job_work_proof_path(&request_id)
        .ok_or_else(|| errors::not_found("proof artifact missing on disk"))?;

    Ok(crate::http::files::proof_file(proof_path).await)
}
