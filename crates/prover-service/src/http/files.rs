use std::path::PathBuf;
use warp::http::StatusCode;
use warp::Reply;

pub async fn proof_file(path: PathBuf) -> warp::reply::Response {
    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            let content_type = match path.extension().and_then(|ext| ext.to_str()) {
                Some("json") => "application/json",
                _ => "application/octet-stream",
            };
            warp::reply::with_header(
                warp::reply::with_status(bytes, StatusCode::OK),
                "content-type",
                content_type,
            )
            .into_response()
        }
        Err(e) => crate::http::replies::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "FILE_READ_FAILED",
            format!("failed to read proof file: {e}"),
        ),
    }
}
