#![allow(clippy::result_large_err)]
use bankai_hints::types::os::BankaiBlockBundleCairo;
use cairo_runner::run_stwo;
use clap::Parser;
use std::{path::Path, path::PathBuf};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_path: PathBuf,

    #[arg(long, conflicts_with = "pie")]
    prove: bool,

    #[arg(long, conflicts_with = "prove")]
    pie: bool,
}

fn main() {
    // Initialize tracing for terminal output
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input_path).unwrap();
    let input: BankaiBlockBundleCairo = serde_json::from_str(&input_str).unwrap();

    let output_dir: &'static str = "output/";
    let log_level: &'static str = "debug";

    let program_path = "cairo/build/main.json";
    let result = run_stwo(
        program_path,
        input,
        log_level,
        output_dir,
        args.prove,
        args.pie,
    );
    let result = result.unwrap();
    if let Some(pie) = result {
        pie.write_zip_file(&Path::new(output_dir).join("pie.zip"), true)
            .unwrap();
        println!("Pie generated successfully");
    }
}
