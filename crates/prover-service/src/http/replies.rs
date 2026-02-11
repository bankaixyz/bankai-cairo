use serde::Serialize;
use warp::http::StatusCode;
use warp::Reply;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error_code: String,
    error_message: String,
}

pub fn json_error(
    status: StatusCode,
    error_code: impl Into<String>,
    error_message: impl Into<String>,
) -> warp::reply::Response {
    let body = ErrorBody {
        error_code: error_code.into(),
        error_message: error_message.into(),
    };
    warp::reply::with_status(warp::reply::json(&body), status).into_response()
}
