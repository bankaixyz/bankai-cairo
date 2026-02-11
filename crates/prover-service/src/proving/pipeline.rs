use crate::app::{now_ms, AppState};
use crate::errors::{ProverError, ProverErrorCode};
use crate::types::{JobRecord, JobStage, JobStatus, SUPPORTED_PAYLOAD_TYPE};
use bankai_hints::types::os::BankaiBlockBundleCairo;
use std::path::PathBuf;
use std::time::Instant;
use cairo_air::utils::ProofFormat;
use tracing::{error, info, instrument};

const JOB_STACK_BYTES: usize = 64 * 1024 * 1024;

#[instrument(skip_all, fields(request_id = %request_id))]
pub async fn run_job(state: AppState, request_id: String) -> Result<(), String> {
    let Some(mut job) = state
        .fs
        .read_job(&request_id)
        .await
        .map_err(|e| format!("failed to read job state: {e}"))?
    else {
        return Ok(());
    };

    if matches!(job.status, JobStatus::Succeeded | JobStatus::Failed) {
        return Ok(());
    }

    let run_started_at = Instant::now();
    info!("starting trace generation");

    // Reset work dir (best-effort) to avoid mixing artifacts across attempts.
    let work_dir = state.fs.job_work_dir(&request_id);
    let _ = tokio::fs::remove_dir_all(&work_dir).await;
    tokio::fs::create_dir_all(&work_dir)
        .await
        .map_err(|e| format!("failed to create work dir: {e}"))?;

    update_job(&state, &mut job, JobStatus::Running, JobStage::Decoding, None, None).await;

    let req = match state.fs.read_request(&request_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            set_end_to_end_ms(&mut job, &run_started_at);
            fail_job(
                &state,
                &mut job,
                ProverError {
                    stage: JobStage::Decoding,
                    code: ProverErrorCode::StateCorrupt,
                    message: "request.json missing on disk".to_string(),
                },
            )
            .await;
            return Ok(());
        }
        Err(e) => {
            set_end_to_end_ms(&mut job, &run_started_at);
            fail_job(
                &state,
                &mut job,
                ProverError {
                    stage: JobStage::Decoding,
                    code: ProverErrorCode::StateCorrupt,
                    message: format!("failed to read request.json: {e}"),
                },
            )
            .await;
            return Ok(());
        }
    };

    if req.payload_type != SUPPORTED_PAYLOAD_TYPE {
        set_end_to_end_ms(&mut job, &run_started_at);
        fail_job(
            &state,
            &mut job,
            ProverError {
                stage: JobStage::Decoding,
                code: ProverErrorCode::DecodingFailed,
                message: format!("unsupported payload_type: {}", req.payload_type),
            },
        )
        .await;
        return Ok(());
    }

    let bundle: BankaiBlockBundleCairo = match serde_json::from_value(req.payload_json) {
        Ok(v) => v,
        Err(e) => {
            set_end_to_end_ms(&mut job, &run_started_at);
            fail_job(
                &state,
                &mut job,
                ProverError {
                    stage: JobStage::Decoding,
                    code: ProverErrorCode::DecodingFailed,
                    message: format!("payload_json decoding failed: {e}"),
                },
            )
            .await;
            return Ok(());
        }
    };

    let trace_gen_started_at = Instant::now();
    update_job(&state, &mut job, JobStatus::Running, JobStage::TraceGen, None, None).await;
    if let Err(e) = trace_gen(&state, bundle, &work_dir).await {
        job.trace_gen_ms = Some(elapsed_ms(&trace_gen_started_at));
        set_end_to_end_ms(&mut job, &run_started_at);
        fail_job(
            &state,
            &mut job,
            ProverError {
                stage: JobStage::TraceGen,
                code: ProverErrorCode::TraceGenFailed,
                message: e,
            },
        )
        .await;
        return Ok(());
    }
    job.trace_gen_ms = Some(elapsed_ms(&trace_gen_started_at));

    info!(
        trace_gen_ms = job.trace_gen_ms.unwrap_or_default(),
        "trace generation done; starting proving"
    );
    let proving_started_at = Instant::now();
    update_job(&state, &mut job, JobStatus::Running, JobStage::Proving, None, None).await;
    let proof_path = match prove(&state, &work_dir).await {
        Ok(path) => path,
        Err(e) => {
            job.proving_ms = Some(elapsed_ms(&proving_started_at));
            set_end_to_end_ms(&mut job, &run_started_at);
            fail_job(
                &state,
                &mut job,
                ProverError {
                    stage: JobStage::Proving,
                    code: ProverErrorCode::ProvingFailed,
                    message: e,
                },
            )
            .await;
            return Ok(());
        }
    };
    job.proving_ms = Some(elapsed_ms(&proving_started_at));
    info!(
        proving_ms = job.proving_ms.unwrap_or_default(),
        proof_path = %proof_path.display(),
        "proving done"
    );
    if !proof_path.exists() {
        set_end_to_end_ms(&mut job, &run_started_at);
        fail_job(
            &state,
            &mut job,
            ProverError {
                stage: JobStage::Proving,
                code: ProverErrorCode::ProvingFailed,
                message: format!(
                    "prover reported success but proof file is missing at {}",
                    proof_path.display()
                ),
            },
        )
        .await;
        return Ok(());
    }

    update_job(
        &state,
        &mut job,
        JobStatus::Running,
        JobStage::Persisting,
        None,
        None,
    )
    .await;
    info!("starting proof publish");

    if let Err(e) = state.fs.publish_proof(&job, &proof_path).await {
        set_end_to_end_ms(&mut job, &run_started_at);
        fail_job(
            &state,
            &mut job,
            ProverError {
                stage: JobStage::Persisting,
                code: ProverErrorCode::PersistFailed,
                message: format!("failed to publish proof: {e}"),
            },
        )
        .await;
        return Ok(());
    }
    info!("proof publish done");

    set_end_to_end_ms(&mut job, &run_started_at);
    update_job(
        &state,
        &mut job,
        JobStatus::Succeeded,
        JobStage::Done,
        None,
        None,
    )
    .await;
    info!(
        end_to_end_ms = job.end_to_end_ms.unwrap_or_default(),
        trace_gen_ms = job.trace_gen_ms.unwrap_or_default(),
        proving_ms = job.proving_ms.unwrap_or_default(),
        "job succeeded"
    );

    Ok(())
}

async fn trace_gen(state: &AppState, bundle: BankaiBlockBundleCairo, work_dir: &PathBuf) -> Result<(), String> {
    let program = state.config.program_path.clone();
    let cairo_log_level = state.config.cairo_log_level_str();
    let work_dir_str = work_dir
        .to_str()
        .ok_or_else(|| "invalid work_dir path".to_string())?
        .to_string();
    let program_str = program
        .to_str()
        .ok_or_else(|| "invalid program path".to_string())?
        .to_string();

    tokio::task::spawn_blocking(move || {
        // `cairo_runner::run_stwo` (and/or its drop path) can be stack-hungry on macOS.
        // Run it on a dedicated thread with a larger stack to prevent stack overflow aborts.
        let handle = std::thread::Builder::new()
            .name("prover-trace-gen".to_string())
            .stack_size(JOB_STACK_BYTES)
            .spawn(move || {
                cairo_runner::run_stwo(
                    &program_str,
                    bundle,
                    cairo_log_level,
                    &work_dir_str,
                    false,
                    false,
                )
            })
            .map_err(|e| format!("failed to spawn trace-gen thread: {e}"))?;

        handle
            .join()
            .map_err(|_| "trace-gen thread panicked".to_string())?
            .map(|_| ())
            .map_err(|e| format!("trace generation failed: {e}"))
    })
    .await
    .map_err(|e| format!("trace generation task join failed: {e}"))?
    // The blocking task returns `Result<(), String>`; propagate it directly.
}

async fn prove(_state: &AppState, work_dir: &PathBuf) -> Result<PathBuf, String> {
    let pub_json = work_dir.join("pub.json");
    let priv_json = work_dir.join("priv.json");
    let pub_json2 = pub_json.clone();
    let priv_json2 = priv_json.clone();

    tokio::task::spawn_blocking(move || {
        // Proving can also be stack-hungry depending on backend/config.
        let handle = std::thread::Builder::new()
            .name("prover-proving".to_string())
            .stack_size(JOB_STACK_BYTES)
            .spawn(move || {
                bankai_stwo_prover::generate_proof(
                    &pub_json2,
                    &priv_json2,
                    Some(false),
                    Some(ProofFormat::Binary),
                )
            })
            .map_err(|e| format!("failed to spawn proving thread: {e}"))?;

        handle
            .join()
            .map_err(|_| "proving thread panicked".to_string())?
            .map(|proof_path| proof_path)
            .map_err(|e| format!("proving failed: {e}"))
    })
    .await
    .map_err(|e| format!("proving task join failed: {e}"))?
    // The blocking task returns `Result<PathBuf, String>`; propagate it directly.
}

async fn update_job(
    state: &AppState,
    job: &mut JobRecord,
    status: JobStatus,
    stage: JobStage,
    error_code: Option<String>,
    error_message: Option<String>,
) {
    job.status = status;
    job.stage = stage;
    job.error_code = error_code;
    job.error_message = error_message;
    job.updated_at_ms = now_ms();

    // Persist first, then broadcast best-effort.
    match state.fs.write_job(job).await {
        Ok(()) => state.broadcast_job(job).await,
        Err(e) => {
            error!(
                request_id = %job.request_id,
                status = ?job.status,
                stage = ?job.stage,
                error = %e,
                "failed to persist job state"
            );
        }
    }
}

async fn fail_job(state: &AppState, job: &mut JobRecord, err: ProverError) {
    update_job(
        state,
        job,
        JobStatus::Failed,
        err.stage,
        Some(err.code.to_string()),
        Some(err.message.clone()),
    )
    .await;
}

fn elapsed_ms(started_at: &Instant) -> u64 {
    started_at.elapsed().as_millis() as u64
}

fn set_end_to_end_ms(job: &mut JobRecord, run_started_at: &Instant) {
    job.end_to_end_ms = Some(elapsed_ms(run_started_at));
}
