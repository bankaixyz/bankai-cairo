use crate::app::AppState;
use crate::types::{ClientWsMessage, JobStage, JobStatus, ServerWsMessage};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, instrument, warn};
use warp::ws::{Message, WebSocket};

#[instrument(skip_all)]
pub async fn handle_ws(ws: WebSocket, state: AppState) {
    let (mut tx, mut rx) = ws.split();

    // Expect a subscribe message first.
    let first = match rx.next().await {
        Some(Ok(m)) => m,
        Some(Err(e)) => {
            warn!("ws receive error: {e}");
            return;
        }
        None => return,
    };

    let request_id = match parse_subscribe(first) {
        Ok(r) => r,
        Err(msg) => {
            let _ = tx
                .send(Message::text(msg))
                .await;
            return;
        }
    };

    // Send current state immediately.
    match state.fs.read_job(&request_id).await {
        Ok(Some(job)) => {
            send_status(&mut tx, ServerWsMessage::status_from_job(&job)).await;
        }
        Ok(None) => {
            let msg = ServerWsMessage::Status {
                request_id: request_id.clone(),
                status: JobStatus::Failed,
                stage: JobStage::Queued,
                error_code: Some("UNKNOWN_REQUEST_ID".to_string()),
                error_message: Some("unknown request_id".to_string()),
            };
            send_status(&mut tx, msg).await;
            return;
        }
        Err(e) => {
            let msg = ServerWsMessage::Status {
                request_id: request_id.clone(),
                status: JobStatus::Failed,
                stage: JobStage::Queued,
                error_code: Some("STATE_READ_FAILED".to_string()),
                error_message: Some(format!("failed to read job state: {e}")),
            };
            send_status(&mut tx, msg).await;
            return;
        }
    }

    let mut updates = state.registry.subscribe(&request_id).await;
    info!(request_id, "ws subscribed");

    loop {
        tokio::select! {
            msg = updates.recv() => {
                match msg {
                    Ok(m) => {
                        send_status(&mut tx, m).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Best-effort channel; client can poll.
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            incoming = rx.next() => {
                match incoming {
                    Some(Ok(_)) => {
                        // Ignore all messages after subscribe.
                        continue;
                    }
                    Some(Err(_)) => break,
                    None => break,
                }
            }
        }
    }
}

fn parse_subscribe(first: Message) -> Result<String, String> {
    if !first.is_text() {
        return Err(r#"{"type":"status","status":"failed","stage":"queued","error_code":"BAD_WS_MESSAGE","error_message":"expected text subscribe message"}"#.to_string());
    }
    let text = first.to_str().map_err(|_| {
        r#"{"type":"status","status":"failed","stage":"queued","error_code":"BAD_WS_MESSAGE","error_message":"invalid utf-8"}"#.to_string()
    })?;

    let msg: ClientWsMessage = serde_json::from_str(text).map_err(|e| {
        format!(
            r#"{{"type":"status","status":"failed","stage":"queued","error_code":"BAD_WS_MESSAGE","error_message":"failed to parse subscribe message: {e}"}}"#
        )
    })?;
    match msg {
        ClientWsMessage::Subscribe { request_id } => Ok(request_id),
    }
}

async fn send_status(tx: &mut (impl futures_util::Sink<Message, Error = warp::Error> + Unpin), msg: ServerWsMessage) {
    match serde_json::to_string(&msg) {
        Ok(s) => {
            let _ = tx.send(Message::text(s)).await;
        }
        Err(_) => {
            // ignore
        }
    }
}
