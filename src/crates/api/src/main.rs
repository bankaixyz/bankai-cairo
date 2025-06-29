use bankai_core::{db::Database, BankaiClient};
use dotenv::from_filename;
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use warp::{reply::json, Filter, Rejection, Reply};

type Db = Arc<Database>;

#[tokio::main]
async fn main() {
    from_filename(".env.local").ok();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("🔌 Initializing Bankai client...");
    let bankai = BankaiClient::new(false).await;
    let db = Arc::new(bankai.db);

    info!("🚀 Starting Bankai API server at 127.0.0.1:3030");

    let api = epochs_route(db.clone())
        .or(proof_route(db.clone()))
        .or(get_beacon_route(db.clone()))
        .or(get_execution_route(db));
    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

// GET /epochs
fn epochs_route(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("epochs")
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_all_epoch_updates)
}

// GET /proofs/:id
fn proof_route(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("proofs" / i64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_proof)
}

// GET /beacon/:height
fn get_beacon_route(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("beacon" / u64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_proof_by_beacon_height)
}

// GET /execution/:height
fn get_execution_route(db: Db) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("execution" / u64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(get_proof_by_execution_height)
}

async fn get_all_epoch_updates(db: Db) -> Result<impl Reply, Rejection> {
    match db.get_all_epoch_updates().await {
        Ok(epochs) => Ok(json(&epochs)),
        Err(e) => {
            error!("Error fetching epoch updates: {}", e);
            Err(warp::reject::custom(DatabaseError))
        }
    }
}

async fn get_proof(id: i64, db: Db) -> Result<impl Reply, Rejection> {
    match db.get_proof(id).await {
        Ok(Some(proof)) => {
            let proof_json: serde_json::Value = serde_json::from_str(&proof.proof)
                .map_err(|_| warp::reject::custom(JsonParseError))?;
            Ok(json(&proof_json))
        }
        Ok(None) => Err(warp::reject::not_found()),
        Err(e) => {
            error!("Error fetching proof {}: {}", id, e);
            Err(warp::reject::custom(DatabaseError))
        }
    }
}

async fn get_proof_by_beacon_height(height: u64, db: Db) -> Result<impl Reply, Rejection> {
    match db.get_proof_by_beacon_height(height).await {
        Ok(Some(proof)) => {
            let proof_json: serde_json::Value = serde_json::from_str(&proof.proof)
                .map_err(|_| warp::reject::custom(JsonParseError))?;
            Ok(json(&proof_json))
        }
        Ok(None) => Err(warp::reject::not_found()),
        Err(e) => {
            error!("Error fetching proof by beacon height {}: {}", height, e);
            Err(warp::reject::custom(DatabaseError))
        }
    }
}

async fn get_proof_by_execution_height(height: u64, db: Db) -> Result<impl Reply, Rejection> {
    match db.get_proof_by_execution_height(height).await {
        Ok(Some(proof)) => {
            let proof_json: serde_json::Value = serde_json::from_str(&proof.proof)
                .map_err(|_| warp::reject::custom(JsonParseError))?;
            Ok(json(&proof_json))
        }
        Ok(None) => Err(warp::reject::not_found()),
        Err(e) => {
            error!("Error fetching proof by execution height {}: {}", height, e);
            Err(warp::reject::custom(DatabaseError))
        }
    }
}

#[derive(Debug)]
struct DatabaseError;
impl warp::reject::Reject for DatabaseError {}

#[derive(Debug)]
struct JsonParseError;
impl warp::reject::Reject for JsonParseError {}

// TODO: Implement the API
