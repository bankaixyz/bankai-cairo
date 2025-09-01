use warp::{Filter, Reply, Rejection, http::StatusCode};
use std::path::Path;
use std::time::Duration;
use bankai_hints::types::StoneCircuitLayoutCairo;
use cairo_runner::run;

#[tokio::main]
async fn main() {
    let generate_pie = warp::path("generate-pie")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_generate_pie);

    let routes = generate_pie
        .with(warp::cors().allow_any_origin());

    println!("Starting server on http://localhost:3030");
    println!("Request timeout: 5 minutes");
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn handle_generate_pie(input: StoneCircuitLayoutCairo) -> Result<Box<dyn Reply>, Rejection> {
    // Set a 5-minute timeout for the operation
    let timeout_duration = Duration::from_secs(300); // 5 minutes
    
    match tokio::time::timeout(timeout_duration, generate_pie_internal(input)).await {
        Ok(Ok(zip_data)) => {
            let timestamp = chrono::Utc::now().timestamp();
            let filename = format!("pie_{timestamp}.zip");
            
            let reply = warp::reply::with_header(
                warp::reply::with_header(
                    warp::reply::with_status(zip_data, StatusCode::OK),
                    "content-type", "application/zip"
                ),
                "content-disposition", 
                &format!("attachment; filename=\"{filename}\"")
            );
            
            Ok(Box::new(reply))
        }
        Ok(Err(e)) => {
            let response = serde_json::json!({
                "status": "error",
                "message": format!("Failed to generate PIE: {}", e)
            });
            let reply = warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::INTERNAL_SERVER_ERROR
            );
            Ok(Box::new(reply))
        }
        Err(_) => {
            let response = serde_json::json!({
                "status": "error",
                "message": "PIE generation timed out after 5 minutes"
            });
            let reply = warp::reply::with_status(
                warp::reply::json(&response),
                StatusCode::REQUEST_TIMEOUT
            );
            Ok(Box::new(reply))
        }
    }
}

async fn generate_pie_internal(input: StoneCircuitLayoutCairo) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let program_path = "cairo/build/bankai_stone.json";
    let output_dir = "output/";
    
    // Generate timestamp for unique filename
    let timestamp = chrono::Utc::now().timestamp();
    let filename = format!("pie_{timestamp}.zip");
    let output_path = Path::new(output_dir).join(&filename);
    
    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;
    
    // Run the PIE generation in a blocking task to avoid blocking the async runtime
    let pie = tokio::task::spawn_blocking(move || {
        run(program_path, input)
    }).await??;
    
    // Write the PIE to zip file
    pie.write_zip_file(&output_path, true)?;
    
    // Read the zip file and return its contents
    let zip_data = std::fs::read(&output_path)?;
    
    // Clean up the temporary file
    std::fs::remove_file(&output_path).ok(); // Ignore errors on cleanup
    
    Ok(zip_data)
}