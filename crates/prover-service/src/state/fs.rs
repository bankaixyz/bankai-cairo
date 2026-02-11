use crate::app::{now_ms, AppState};
use crate::types::{JobRecord, JobStage, JobStatus, SubmitProofRequest};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct Fs {
    data_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum FsError {
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("json failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("request_id already exists")]
    AlreadyExists,
}

impl Fs {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub async fn ensure_base_dirs(&self) -> Result<(), FsError> {
        tokio::fs::create_dir_all(self.jobs_dir()).await?;
        tokio::fs::create_dir_all(self.proofs_dir()).await?;
        Ok(())
    }

    pub fn jobs_dir(&self) -> PathBuf {
        self.data_dir.join("jobs")
    }

    pub fn proofs_dir(&self) -> PathBuf {
        self.data_dir.join("proofs")
    }

    pub fn job_dir(&self, request_id: &str) -> PathBuf {
        self.jobs_dir()
            .join(crate::state::util::sanitize_request_id_for_path(request_id))
    }

    pub fn job_request_path(&self, request_id: &str) -> PathBuf {
        self.job_dir(request_id).join("request.json")
    }

    pub fn job_record_path(&self, request_id: &str) -> PathBuf {
        self.job_dir(request_id).join("job.json")
    }

    pub fn job_work_dir(&self, request_id: &str) -> PathBuf {
        self.job_dir(request_id).join("work")
    }

    pub fn job_work_proof_path(&self, request_id: &str) -> PathBuf {
        self.job_work_dir(request_id).join("proof.json")
    }

    pub fn proof_index_dir(&self, request_id: &str) -> PathBuf {
        self.proofs_dir()
            .join(crate::state::util::sanitize_request_id_for_path(request_id))
    }

    pub async fn persist_new_job(
        &self,
        job: &JobRecord,
        request: &SubmitProofRequest,
    ) -> Result<(), FsError> {
        self.ensure_base_dirs().await?;

        let job_dir = self.job_dir(&job.request_id);
        match tokio::fs::create_dir(&job_dir).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(FsError::AlreadyExists);
            }
            Err(e) => return Err(FsError::Io(e)),
        }
        tokio::fs::create_dir_all(self.job_work_dir(&job.request_id)).await?;

        let request_pretty = serde_json::to_string_pretty(request)?;
        crate::state::atomic::write_atomic(
            &self.job_request_path(&job.request_id),
            request_pretty.as_bytes(),
        )
        .await?;

        self.write_job(job).await?;

        // Ensure directory exists (should already).
        tokio::fs::create_dir_all(&job_dir).await?;

        Ok(())
    }

    pub async fn read_job(&self, request_id: &str) -> Result<Option<JobRecord>, FsError> {
        let path = self.job_record_path(request_id);
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(Some(serde_json::from_slice::<JobRecord>(&bytes)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(FsError::Io(e)),
        }
    }

    pub async fn write_job(&self, job: &JobRecord) -> Result<(), FsError> {
        let bytes = serde_json::to_vec_pretty(job)?;
        crate::state::atomic::write_atomic(&self.job_record_path(&job.request_id), &bytes).await?;
        Ok(())
    }

    pub async fn read_request(
        &self,
        request_id: &str,
    ) -> Result<Option<SubmitProofRequest>, FsError> {
        let path = self.job_request_path(request_id);
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(Some(serde_json::from_slice::<SubmitProofRequest>(&bytes)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(FsError::Io(e)),
        }
    }

    pub async fn publish_proof(
        &self,
        job: &JobRecord,
        proof_path: &Path,
    ) -> Result<(), FsError> {
        let idx_dir = self.proof_index_dir(&job.request_id);
        tokio::fs::create_dir_all(&idx_dir).await?;

        let proof_dst = idx_dir.join("proof.json");
        crate::state::atomic::copy_atomic(proof_path, &proof_dst).await?;

        let meta = serde_json::json!({
            "request_id": job.request_id,
            "idempotency_key": job.idempotency_key,
            "published_at_ms": now_ms(),
        });
        let meta_bytes = serde_json::to_vec_pretty(&meta)?;
        crate::state::atomic::write_atomic(&idx_dir.join("meta.json"), &meta_bytes).await?;

        let link = serde_json::json!({
            "job_dir": self.job_dir(&job.request_id).to_string_lossy(),
            "work_proof": self.job_work_proof_path(&job.request_id).to_string_lossy(),
        });
        let link_bytes = serde_json::to_vec_pretty(&link)?;
        crate::state::atomic::write_atomic(&idx_dir.join("link.json"), &link_bytes).await?;

        Ok(())
    }

    pub async fn bootstrap_fail_unfinished(&self, state: AppState) -> Result<u64, FsError> {
        self.ensure_base_dirs().await?;

        let mut failed_unfinished = 0u64;
        let mut dir = match tokio::fs::read_dir(self.jobs_dir()).await {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(FsError::Io(e)),
        };

        while let Ok(Some(entry)) = dir.next_entry().await {
            let entry_path = entry.path();
            let job_path = entry_path.join("job.json");
            let bytes = match tokio::fs::read(&job_path).await {
                Ok(b) => b,
                Err(e) => {
                    warn!(?job_path, "failed to read job.json: {e}");
                    continue;
                }
            };

            let mut job: JobRecord = match serde_json::from_slice(&bytes) {
                Ok(j) => j,
                Err(e) => {
                    warn!(?job_path, "failed to parse job.json: {e}");
                    continue;
                }
            };

            // Migrate legacy hashed job directories to request_id-based paths.
            let expected_dir = self.job_dir(&job.request_id);
            if entry_path != expected_dir && !expected_dir.exists() {
                if let Err(e) = tokio::fs::rename(&entry_path, &expected_dir).await {
                    warn!(
                        from = %entry_path.display(),
                        to = %expected_dir.display(),
                        "failed to migrate legacy job dir: {e}"
                    );
                }
            }

            state.registry.ensure(&job.request_id).await;

            if matches!(job.status, JobStatus::Queued | JobStatus::Running) {
                let proof = self.job_work_proof_path(&job.request_id);
                if proof.exists() {
                    job.status = JobStatus::Succeeded;
                    job.stage = JobStage::Done;
                    job.updated_at_ms = now_ms();
                    self.write_job(&job).await?;
                    state.broadcast_job(&job).await;
                    continue;
                }

                job.status = JobStatus::Failed;
                job.error_code = Some("PROVER_INTERRUPTED".to_string());
                job.error_message = Some(
                    "job was interrupted before completion; retries are handled externally"
                        .to_string(),
                );
                job.updated_at_ms = now_ms();
                self.write_job(&job).await?;
                state.broadcast_job(&job).await;
                failed_unfinished += 1;
            } else {
                state.broadcast_job(&job).await;
            }
        }

        info!(failed_unfinished, "marked unfinished jobs as failed");
        Ok(failed_unfinished)
    }
}
