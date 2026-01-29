#![allow(clippy::result_large_err)]
use bankai_hints::types::CircuitRunDataCairo;
use cairo_runner::{run, run_stwo};
use clap::Parser;
use serde_json::json;
use std::{path::Path, path::PathBuf};
use std::{fs::OpenOptions, io::Write};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_path: PathBuf,

    #[arg(long, conflicts_with = "stone", required_unless_present = "stone")]
    stwo: bool,

    #[arg(long, conflicts_with = "stwo", required_unless_present = "stwo")]
    stone: bool,

    #[arg(long, requires = "stwo", conflicts_with = "pie")]
    prove: bool,

    #[arg(long, requires = "stwo", conflicts_with = "prove")]
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
    let input: CircuitRunDataCairo = serde_json::from_str(&input_str).unwrap();

    let output_dir: &'static str = "output/";
    let log_level: &'static str = "debug";

    if args.stwo {
        let program_path = "cairo/build/bankai_stwo.json";
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
    } else {
        let program_path = "cairo/build/bankai_stone.json";
        let result = run(program_path, input, log_level);
        let pie = result.unwrap();
        pie.write_zip_file(&Path::new(output_dir).join("pie.zip"), true)
            .unwrap();
        println!("Pie generated successfully");
    }
}
