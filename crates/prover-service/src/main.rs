mod app;
mod config;
mod errors;
mod http;
mod proving;
mod state;
mod types;
mod ws;

use crate::app::App;
use crate::config::Config;
use clap::Parser;
use rayon::ThreadPoolBuilder;
use std::net::SocketAddr;
use tracing::{info, Level};
use warp::Filter;

#[tokio::main]
async fn main() {
    let config = Config::parse().finalize();
    init_tracing(&config);
    init_rayon();

    info!(?config, "starting prover-service");

    let app = App::new(config).await;
    let routes = http::routes::routes(app.state.clone())
        .with(warp::trace::request())
        .recover(errors::recover);

    let bind: SocketAddr = app.state.config.bind;
    info!(%bind, "listening");

    warp::serve(routes)
        .bind(bind)
        .await
        .graceful(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
            info!("received Ctrl+C, shutting down");
        })
        .run()
        .await;
}

fn init_tracing(config: &Config) {
    let max_level = match config.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(max_level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn init_rayon() {
    let num_threads = num_cpus::get();
    std::env::set_var("RAYON_NUM_THREADS", num_threads.to_string());
    if ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .is_ok()
    {
        info!(num_threads, "configured rayon global thread pool");
    } else {
        info!(num_threads, "rayon global thread pool already configured");
    }
}
