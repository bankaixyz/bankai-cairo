#![allow(clippy::result_large_err)]
use bankai_hints::types::StoneCircuitLayoutCairo;
use cairo_runner::run;
use clap::Parser;
use std::{path::Path, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_path: PathBuf,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input_path).unwrap();
    let input: StoneCircuitLayoutCairo = serde_json::from_str(&input_str).unwrap();

    generate_pie(input);
    println!("Pie generated successfully");
}

fn generate_pie(input: StoneCircuitLayoutCairo) {
    let program_path = "cairo/build/bankai_stone.json";
    let output_dir: &'static str = "output/";
    let log_level: &'static str = "debug";

    let pie = run(program_path, input, log_level).unwrap();

    pie.write_zip_file(&Path::new(output_dir).join("pie.zip"), true)
        .unwrap();
}
