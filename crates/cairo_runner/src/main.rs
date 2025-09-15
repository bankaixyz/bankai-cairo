#![allow(clippy::result_large_err)]
use bankai_hints::types::StoneCircuitLayoutCairo;
use cairo_runner::{run, run_stwo};
use clap::Parser;
use std::{path::Path, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_path: PathBuf,

    #[arg(long, conflicts_with = "stone", required_unless_present = "stone")]
    stwo: bool,

    #[arg(long, conflicts_with = "stwo", required_unless_present = "stwo")]
    stone: bool,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input_path).unwrap();
    let input: StoneCircuitLayoutCairo = serde_json::from_str(&input_str).unwrap();

    let output_dir: &'static str = "output/";
    let log_level: &'static str = "debug";

    if args.stwo {
        let program_path = "cairo/build/bankai_stwo.json";
        run_stwo(program_path, input, log_level, output_dir).unwrap();
        println!("STWO artifacts generated successfully");
    } else {
        let program_path = "cairo/build/bankai_stone.json";
        let pie = run(program_path, input, log_level).unwrap();
        pie.write_zip_file(&Path::new(output_dir).join("pie.zip"), true)
            .unwrap();
        println!("Pie generated successfully");
    }
}
