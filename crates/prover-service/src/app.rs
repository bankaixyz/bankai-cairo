use crate::config::Config;
use crate::state::{Fs, Registry};
use crate::types::{JobRecord, ServerWsMessage};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;

pub type AppState = Arc<AppStateInner>;

#[derive(Debug)]
pub struct AppStateInner {
    pub config: Config,
    pub fs: Fs,
    pub registry: Registry,
    pub semaphore: Arc<Semaphore>,
}

pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn new(config: Config) -> Self {
        let fs = Fs::new(config.data_dir.clone());
        fs.ensure_base_dirs()
            .await
            .expect("failed to create base dirs");
        let registry = Registry::new();
        // No internal queueing: the service runs at most one proving job at a time.
        let semaphore = Arc::new(Semaphore::new(1));

        let state: AppState = Arc::new(AppStateInner {
            config,
            fs,
            registry,
            semaphore,
        });

        // Startup: unfinished jobs are marked failed; retries are handled externally.
        let failed_unfinished = state
            .fs
            .bootstrap_fail_unfinished(state.clone())
            .await
            .expect("bootstrap failed");
        info!(failed_unfinished, "bootstrap complete");

        Self { state }
    }
}

pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl AppStateInner {
    pub async fn broadcast_job(&self, job: &JobRecord) {
        let sender = self.registry.ensure(&job.request_id).await;
        let _ = sender.send(ServerWsMessage::status_from_job(job));
    }
}
