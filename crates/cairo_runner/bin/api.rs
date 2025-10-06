use bankai_hints::types::CircuitRunDataCairo;
use cairo_runner::{run, run_stwo};
use clap::Parser;
use rayon::ThreadPoolBuilder;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use warp::{http::StatusCode, Filter, Rejection, Reply};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in Docker mode
    #[arg(long, default_value_t = false)]
    docker: bool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    // Configure Rayon to use all available CPU cores for prover-heavy workloads
    let num_threads = num_cpus::get();
    std::env::set_var("RAYON_NUM_THREADS", num_threads.to_string());
    if ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .is_ok()
    {
        info!("Configured Rayon global thread pool with {num_threads} threads");
    } else {
        info!("Rayon global thread pool already configured");
    }
    info!(
        "Running in {} mode",
        if args.docker { "Docker" } else { "local" }
    );

    let docker_flag = Arc::new(args.docker);
    let docker_flag_filter = warp::any().map(move || docker_flag.clone());

    let generate_pie = warp::path("stone")
        .and(warp::path("execute"))
        .and(warp::post())
        .and(warp::body::json())
        .and(docker_flag_filter.clone())
        .and_then(handle_generate_pie);

    let generate_proof = warp::path("stwo")
        .and(warp::path("prove"))
        .and(warp::post())
        .and(warp::body::json())
        .and(docker_flag_filter)
        .and_then(handle_generate_proof);

    let routes = generate_pie
        .or(generate_proof)
        .with(warp::cors().allow_any_origin())
        .with(warp::trace::request());

    info!("Starting server on http://localhost:3030");
    info!("Request timeout: 5 minutes");

    let bind_addr = if args.docker {
        ([0, 0, 0, 0], 3030)
    } else {
        ([127, 0, 0, 1], 3030)
    };

    // Create the shutdown signal future
    warp::serve(routes)
        .bind(bind_addr)
        .await
        .graceful(async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
            info!("Received Ctrl+C, shutting down...");
        })
        .run()
        .await;
}

async fn handle_generate_pie(
    input: CircuitRunDataCairo,
    is_docker: Arc<bool>,
) -> Result<Box<dyn Reply>, Rejection> {
    // Set a 5-minute timeout for the operation
    info!("Generating PIE...");
    let timeout_duration = Duration::from_secs(300); // 5 minutes
    info!("Timeout duration: {:?}", timeout_duration);
    match tokio::time::timeout(timeout_duration, generate_pie_internal(input, *is_docker)).await {
        Ok(Ok(zip_data)) => {
            info!("PIE generated successfully");
            let timestamp = chrono::Utc::now().timestamp();
            let filename = format!("pie_{timestamp}.zip");

            let reply = warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_status(zip_data, StatusCode::OK),
                    "content-type",
                    "application/zip",
                ),
                "content-disposition",
                &format!("attachment; filename=\"{filename}\""),
            );

            Ok(Box::new(reply))
        }
        Ok(Err(e)) => {
            info!("Failed to generate PIE: {e}");
            let response = serde_json::json!({
                "status": "error",
                "message": format!("Failed to generate PIE: {}", e)
            });
            let reply = warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
            Ok(Box::new(reply))
        }
        Err(_) => {
            info!("PIE generation timed out after 5 minutes");
            let response = serde_json::json!({
                "status": "error",
                "message": "PIE generation timed out after 5 minutes"
            });
            let reply =
                warp::reply::with_status(warp::reply::json(&response), StatusCode::REQUEST_TIMEOUT);
            Ok(Box::new(reply))
        }
    }
}

async fn generate_pie_internal(
    input: CircuitRunDataCairo,
    is_docker: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (program_path, output_dir, log_level) = if is_docker {
        ("/app/cairo/build/bankai_stone.json", "/app/output/", "info")
    } else {
        ("cairo/build/bankai_stone.json", "output/", "debug")
    };

    // Generate timestamp for unique filename
    let timestamp = chrono::Utc::now().timestamp();
    let filename = format!("pie_{timestamp}.zip");
    let output_path = Path::new(output_dir).join(&filename);

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    // Run the PIE generation in a blocking task to avoid blocking the async runtime
    let pie = tokio::task::spawn_blocking(move || run(program_path, input, log_level)).await??;

    // Write the PIE to zip file
    pie.write_zip_file(&output_path, true)?;

    // Read the zip file and return its contents
    let zip_data = std::fs::read(&output_path)?;

    // Clean up the temporary file
    std::fs::remove_file(&output_path).ok(); // Ignore errors on cleanup

    Ok(zip_data)
}

async fn handle_generate_proof(
    input: CircuitRunDataCairo,
    is_docker: Arc<bool>,
) -> Result<Box<dyn Reply>, Rejection> {
    info!("Generating STWO trace and proof...");
    let start = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(300);
    info!("Timeout duration: {:?}", timeout_duration);
    // Periodic elapsed-time logger while proving runs
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let ticker_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let elapsed = start.elapsed();
                    info!("STWO proving in progress... elapsed: {:.1?}", elapsed);
                }
                _ = rx.recv() => {
                    break;
                }
            }
        }
    });

    let result =
        tokio::time::timeout(timeout_duration, generate_proof_internal(input, *is_docker)).await;
    let _ = tx.send(());
    let _ = ticker_handle.await;
    match result {
        Ok(Ok(proof_data)) => {
            info!("Proof generated successfully in {:.1?}", start.elapsed());
            let timestamp = chrono::Utc::now().timestamp();
            let filename = format!("proof_{timestamp}.json");

            let reply = warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_status(proof_data, StatusCode::OK),
                    "content-type",
                    "application/json",
                ),
                "content-disposition",
                &format!("attachment; filename=\"{filename}\""),
            );

            Ok(Box::new(reply))
        }
        Ok(Err(e)) => {
            info!(
                "Failed to generate proof after {:.1?}: {e}",
                start.elapsed()
            );
            let response = serde_json::json!({
                "status": "error",
                "message": format!("Failed to generate proof: {}", e)
            });
            let reply = warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
            Ok(Box::new(reply))
        }
        Err(_) => {
            info!("Proof generation timed out after {:.1?}", start.elapsed());
            let response = serde_json::json!({
                "status": "error",
                "message": "Proof generation timed out after 5 minutes"
            });
            let reply =
                warp::reply::with_status(warp::reply::json(&response), StatusCode::REQUEST_TIMEOUT);
            Ok(Box::new(reply))
        }
    }
}

async fn generate_proof_internal(
    input: CircuitRunDataCairo,
    is_docker: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (program_path, base_output_dir, log_level) = if is_docker {
        ("/app/cairo/build/bankai_stwo.json", "/app/output/", "info")
    } else {
        ("cairo/build/bankai_stwo.json", "output/", "debug")
    };

    // Unique per-request directory to avoid collisions, and allow cleanup
    let timestamp = chrono::Utc::now().timestamp();
    let subdir = format!("stwo_{timestamp}");
    let output_dir_path = Path::new(base_output_dir).join(&subdir);
    std::fs::create_dir_all(&output_dir_path)?;

    let output_dir_string = output_dir_path
        .to_str()
        .ok_or("Invalid output directory path")?
        .to_string();

    // Run the STWO flow in a blocking task
    tokio::task::spawn_blocking(move || {
        run_stwo(
            program_path,
            input,
            log_level,
            &output_dir_string,
            true,
            false,
        )
    })
    .await??;

    // Read proof.json and return its contents
    let proof_path = output_dir_path.join("proof.json");
    let proof_data = std::fs::read(&proof_path)?;

    // Cleanup temp directory (best-effort)
    std::fs::remove_dir_all(&output_dir_path).ok();

    Ok(proof_data)
}
