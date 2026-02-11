use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct Config {
    /// Bind address for the HTTP server.
    #[arg(long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3031))]
    pub bind: SocketAddr,

    /// Directory for job state and proof artifacts.
    #[arg(long, default_value = "prover-data")]
    pub data_dir: PathBuf,

    /// Compiled Cairo program json (only one program supported).
    #[arg(long, default_value = "cairo/build/main.json")]
    pub program_path: PathBuf,

    /// If set, require `Authorization: Bearer <token>`.
    #[arg(long)]
    pub auth_token: Option<String>,

    /// Tracing log level directive for this crate (e.g. "info", "debug").
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Log level passed into Cairo runner exec scope.
    #[arg(long, value_enum, default_value_t = CairoLogLevel::Info)]
    pub cairo_log_level: CairoLogLevel,
}

impl Config {
    pub fn finalize(mut self) -> Self {
        if self.auth_token.is_none() {
            self.auth_token = std::env::var("PROVER_AUTH_TOKEN").ok();
        }
        self
    }

    pub fn cairo_log_level_str(&self) -> &'static str {
        match self.cairo_log_level {
            CairoLogLevel::Info => "info",
            CairoLogLevel::Debug => "debug",
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CairoLogLevel {
    Info,
    Debug,
}
