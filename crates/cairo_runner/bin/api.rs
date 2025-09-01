use bankai_hints::types::StoneCircuitLayoutCairo;
use cairo_runner::run;
use clap::Parser;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, instrument, Level};
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
    let args = Args::parse();

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!(
        "Running in {} mode",
        if args.docker { "Docker" } else { "local" }
    );

    let docker_flag = Arc::new(args.docker);
    let docker_flag_filter = warp::any().map(move || docker_flag.clone());

    let generate_pie = warp::path("generate-pie")
        .and(warp::post())
        .and(warp::body::json())
        .and(docker_flag_filter)
        .and_then(handle_generate_pie);

    let routes = generate_pie
        .with(warp::cors().allow_any_origin())
        .with(warp::trace::request());

    info!("Starting server on http://localhost:3030");
    info!("Request timeout: 5 minutes");

    let bind_addr = if args.docker {
        ([0, 0, 0, 0], 3030) // Bind to all interfaces in Docker
    } else {
        ([127, 0, 0, 1], 3030) // Bind to localhost only when running locally
    };

    warp::serve(routes).run(bind_addr).await;
}

#[instrument]
async fn handle_generate_pie(
    input: StoneCircuitLayoutCairo,
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

#[instrument]
async fn generate_pie_internal(
    input: StoneCircuitLayoutCairo,
    is_docker: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (program_path, output_dir) = if is_docker {
        ("/app/cairo/build/bankai_stone.json", "/app/output/")
    } else {
        ("cairo/build/bankai_stone.json", "output/")
    };

    // Generate timestamp for unique filename
    let timestamp = chrono::Utc::now().timestamp();
    let filename = format!("pie_{timestamp}.zip");
    let output_path = Path::new(output_dir).join(&filename);

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    // Run the PIE generation in a blocking task to avoid blocking the async runtime
    let pie = tokio::task::spawn_blocking(move || run(program_path, input)).await??;

    // Write the PIE to zip file
    pie.write_zip_file(&output_path, true)?;

    // Read the zip file and return its contents
    let zip_data = std::fs::read(&output_path)?;

    // Clean up the temporary file
    std::fs::remove_file(&output_path).ok(); // Ignore errors on cleanup

    Ok(zip_data)
}
