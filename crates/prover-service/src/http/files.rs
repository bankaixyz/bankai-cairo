use std::path::PathBuf;
use warp::http::StatusCode;
use warp::Reply;

pub async fn json_file(path: PathBuf) -> warp::reply::Response {
    match tokio::fs::read(&path).await {
        Ok(bytes) => warp::reply::with_header(
            warp::reply::with_status(bytes, StatusCode::OK),
            "content-type",
            "application/json",
        )
        .into_response(),
        Err(e) => crate::http::replies::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "FILE_READ_FAILED",
            format!("failed to read proof file: {e}"),
        ),
    }
}

