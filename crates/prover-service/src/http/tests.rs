use crate::app::{AppState, AppStateInner};
use crate::config::{CairoLogLevel, Config};
use crate::errors;
use crate::http::routes;
use crate::state::{Fs, Registry};
use crate::types::{
    JobRecord, JobStage, JobStatus, ServerWsMessage, SubmitMetadata, SubmitProofRequest,
};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use warp::Filter;

async fn test_state(tmp: &tempfile::TempDir) -> AppState {
    let config = Config {
        bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        data_dir: tmp.path().join("prover-data"),
        program_path: PathBuf::from("cairo/build/main.json"),
        auth_token: None,
        log_level: "info".to_string(),
        cairo_log_level: CairoLogLevel::Info,
    };

    let fs = Fs::new(config.data_dir.clone());
    fs.ensure_base_dirs().await.unwrap();

    let registry = Registry::new();

    Arc::new(AppStateInner {
        config,
        fs,
        registry,
        semaphore: Arc::new(Semaphore::new(1)),
    })
}

fn test_filter(
    state: AppState,
) -> impl Filter<Extract = (impl warp::Reply,), Error = std::convert::Infallible> + Clone {
    routes::routes(state).recover(errors::recover)
}

fn sample_request(request_id: &str) -> SubmitProofRequest {
    SubmitProofRequest {
        request_id: request_id.to_string(),
        idempotency_key: format!("block:{request_id}"),
        payload_type: crate::types::SUPPORTED_PAYLOAD_TYPE.to_string(),
        payload_json: json!({}),
        metadata: SubmitMetadata {
            producer_id: "test".to_string(),
            producer_attempt: Some(0),
            max_primary_attempts: Some(1),
        },
    }
}

#[tokio::test]
async fn submit_is_deduped_by_request_id_only() {
    let tmp = tempfile::tempdir().unwrap();
    let state = test_state(&tmp).await;
    let api = test_filter(state);

    let body1 = json!({
        "request_id": "r1",
        "idempotency_key": "block:123",
        "payload_type": "bankai_block_bundle_cairo",
        "payload_json": {},
        "metadata": {
            "producer_id": "test",
            "producer_attempt": 0,
            "max_primary_attempts": 1
        }
    });

    let resp1 = warp::test::request()
        .method("POST")
        .path("/v1/proofs")
        .json(&body1)
        .reply(&api)
        .await;
    assert_eq!(resp1.status(), 202);

    let resp2 = warp::test::request()
        .method("POST")
        .path("/v1/proofs")
        .json(&body1)
        .reply(&api)
        .await;
    assert_eq!(resp2.status(), 202);

    let body2 = json!({
        "request_id": "r1",
        "idempotency_key": "block:123",
        "payload_type": "bankai_block_bundle_cairo",
        "payload_json": {"different": true},
        "metadata": {
            "producer_id": "test",
            "producer_attempt": 0,
            "max_primary_attempts": 1
        }
    });

    let resp3 = warp::test::request()
        .method("POST")
        .path("/v1/proofs")
        .json(&body2)
        .reply(&api)
        .await;
    assert_eq!(resp3.status(), 202);
}

#[tokio::test]
async fn submit_when_busy_returns_503() {
    let tmp = tempfile::tempdir().unwrap();
    let state = test_state(&tmp).await;
    let api = test_filter(state.clone());

    let _permit = state.semaphore.clone().try_acquire_owned().unwrap();

    let body = json!({
        "request_id": "busy-1",
        "idempotency_key": "block:busy-1",
        "payload_type": "bankai_block_bundle_cairo",
        "payload_json": {},
        "metadata": {
            "producer_id": "test",
            "producer_attempt": 0,
            "max_primary_attempts": 1
        }
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v1/proofs")
        .json(&body)
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 503);

    let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
    assert_eq!(body["error_code"], "BUSY");
}

#[tokio::test]
async fn status_unknown_is_404() {
    let tmp = tempfile::tempdir().unwrap();
    let state = test_state(&tmp).await;
    let api = test_filter(state);

    let resp = warp::test::request()
        .method("GET")
        .path("/v1/proofs/nope/status")
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn proof_before_ready_is_409() {
    let tmp = tempfile::tempdir().unwrap();
    let state = test_state(&tmp).await;
    let api = test_filter(state);

    let body = json!({
        "request_id": "r2",
        "idempotency_key": "block:124",
        "payload_type": "bankai_block_bundle_cairo",
        "payload_json": {},
        "metadata": {
            "producer_id": "test",
            "producer_attempt": 0,
            "max_primary_attempts": 1
        }
    });

    let resp1 = warp::test::request()
        .method("POST")
        .path("/v1/proofs")
        .json(&body)
        .reply(&api)
        .await;
    assert_eq!(resp1.status(), 202);

    let resp2 = warp::test::request()
        .method("GET")
        .path("/v1/proofs/r2/proof")
        .reply(&api)
        .await;
    assert_eq!(resp2.status(), 409);
}

#[tokio::test]
async fn proof_download_uses_published_copy_after_work_cleanup() {
    let tmp = tempfile::tempdir().unwrap();
    let state = test_state(&tmp).await;
    let api = test_filter(state.clone());

    let job = JobRecord {
        request_id: "published-proof".to_string(),
        idempotency_key: "block:published-proof".to_string(),
        status: JobStatus::Succeeded,
        stage: JobStage::Done,
        error_code: None,
        error_message: None,
        created_at_ms: 1,
        updated_at_ms: 1,
        end_to_end_ms: None,
        trace_gen_ms: None,
        proving_ms: None,
    };
    state
        .fs
        .persist_new_job(&job, &sample_request("published-proof"))
        .await
        .unwrap();
    tokio::fs::write(
        state.fs.job_work_proof_path("published-proof"),
        b"work-proof",
    )
    .await
    .unwrap();
    tokio::fs::create_dir_all(state.fs.proof_index_dir("published-proof"))
        .await
        .unwrap();
    tokio::fs::write(
        state.fs.proof_index_proof_path("published-proof"),
        b"published-proof",
    )
    .await
    .unwrap();
    state
        .fs
        .delete_job_work_dir("published-proof")
        .await
        .unwrap();

    let resp = warp::test::request()
        .method("GET")
        .path("/v1/proofs/published-proof/proof")
        .reply(&api)
        .await;

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body().as_ref(), b"published-proof");
}

#[tokio::test]
async fn ws_reports_decoding_failure_with_stage() {
    let tmp = tempfile::tempdir().unwrap();
    let state = test_state(&tmp).await;

    let api = test_filter(state.clone());
    let ws_api = routes::routes(state.clone());

    // This is valid submit shape but invalid for BankaiBlockBundleCairo, so it will fail at decoding stage.
    let body = json!({
        "request_id": "r3",
        "idempotency_key": "block:125",
        "payload_type": "bankai_block_bundle_cairo",
        "payload_json": {},
        "metadata": {
            "producer_id": "test",
            "producer_attempt": 0,
            "max_primary_attempts": 1
        }
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v1/proofs")
        .json(&body)
        .reply(&api)
        .await;
    assert_eq!(resp.status(), 202);

    let mut client = warp::test::ws()
        .path("/v1/proofs/ws")
        .handshake(ws_api)
        .await
        .expect("handshake");

    client
        .send_text(r#"{ "type": "subscribe", "request_id": "r3" }"#)
        .await;

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if tokio::time::Instant::now() > deadline {
            panic!("did not receive failed status in time");
        }

        let msg = client.recv().await.expect("recv");
        let text = msg.to_str().unwrap();
        let parsed: ServerWsMessage = serde_json::from_str(text).unwrap();

        let ServerWsMessage::Status {
            status,
            stage,
            error_code,
            error_message,
            ..
        } = parsed;
        if status == JobStatus::Failed {
            assert_eq!(stage, JobStage::Decoding);
            assert!(error_code.unwrap().contains("PROVER_DECODING_FAILED"));
            assert!(error_message
                .unwrap()
                .contains("payload_json decoding failed"));
            break;
        }
    }
}
