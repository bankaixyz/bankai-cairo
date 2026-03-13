use crate::types::ServerWsMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone)]
pub struct Registry {
    inner: Arc<Mutex<HashMap<String, broadcast::Sender<ServerWsMessage>>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn ensure(&self, request_id: &str) -> broadcast::Sender<ServerWsMessage> {
        let mut g = self.inner.lock().await;
        if let Some(s) = g.get(request_id) {
            return s.clone();
        }

        // Small buffer is fine; WS delivery is best-effort and the worker can poll HTTP.
        let (tx, _rx) = broadcast::channel::<ServerWsMessage>(32);
        g.insert(request_id.to_string(), tx.clone());
        tx
    }

    pub async fn subscribe(&self, request_id: &str) -> broadcast::Receiver<ServerWsMessage> {
        self.ensure(request_id).await.subscribe()
    }
}
