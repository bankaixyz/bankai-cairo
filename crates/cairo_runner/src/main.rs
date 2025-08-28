#![allow(clippy::result_large_err)]
use bankai_hints::types::StoneCircuitLayoutCairo;
use cairo_runner::{
    error::Error, hint_processor::CustomHintProcessor,
};
use cairo_vm_base::vm::cairo_vm::{
    cairo_run::{self, cairo_run_program_with_initial_scope},
    types::{exec_scope::ExecutionScopes, layout_name::LayoutName, program::Program},
    vm::runners::cairo_pie::CairoPie,
};
use clap::Parser;
use std::{path::Path, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_path: PathBuf,
}

fn load_program(path: &str) -> Result<Program, Error> {
    // Check if it's an absolute path that doesn't exist, try relative
    let final_path = if path.starts_with('/') && !std::path::Path::new(path).exists() {
        // Try converting absolute path to relative
        let relative_path = path.strip_prefix('/').unwrap_or(path);
        println!("Absolute path not found, trying relative: {relative_path}");
        relative_path
    } else {
        path
    };

    let program_file = std::fs::read(final_path).map_err(Error::IO)?;
    let cairo_run_config = cairo_run::CairoRunConfig {
        allow_missing_builtins: Some(true),
        layout: LayoutName::all_cairo,
        ..Default::default()
    };

    let program = Program::from_bytes(&program_file, Some(cairo_run_config.entrypoint))?;
    println!("Program loaded successfully");
    Ok(program)
}

// pub fn run_stwo(path: &str, input: MmrInput, output_dir: &str) -> Result<(), Error> {
//     let program = load_program(path)?;
//     let cairo_run_config = cairo_run::CairoRunConfig {
//         allow_missing_builtins: None, // Optional
//         layout: LayoutName::all_cairo_stwo,
//         relocate_mem: true,
//         trace_enabled: true,
//         proof_mode: true,
//         ..Default::default()
//     };

//     let mut hint_processor = CustomHintProcessor::new();
//     let mut exec_scopes = ExecutionScopes::new();
//     exec_scopes.insert_value("beacon_input", input);

//     let cairo_runner = cairo_run_program_with_initial_scope(
//         &program,
//         &cairo_run_config,
//         &mut hint_processor,
//         exec_scopes,
//     )?;

//     // tracing::info!("{:?}", cairo_runner.get_execution_resources());

//     generate_stwo_files(&cairo_runner, output_dir)?;
//     Ok(())
// }

pub fn run(path: &str, input: StoneCircuitLayoutCairo) -> Result<CairoPie, Error> {
    let program = load_program(path)?;
    let cairo_run_config = cairo_run::CairoRunConfig {
        allow_missing_builtins: Some(true),
        layout: LayoutName::all_cairo,
        ..Default::default()
    };
    let mut hint_processor = CustomHintProcessor::new();
    let mut exec_scopes = ExecutionScopes::new();
    exec_scopes.insert_value("inputs", input.input);
    exec_scopes.insert_value("outputs", input.output);
    exec_scopes.insert_value("program_object", program.clone());

    let cairo_runner = cairo_run_program_with_initial_scope(
        &program,
        &cairo_run_config,
        &mut hint_processor,
        exec_scopes,
    )?;

    println!("Resources: {:?}", cairo_runner.get_execution_resources());

    let pie = cairo_runner.get_cairo_pie()?;
    Ok(pie)
}

// fn generate_stwo_files(cairo_runner: &CairoRunner, output_dir: &str) -> Result<(), Error> {
//     std::fs::create_dir_all(output_dir)?;

//     let memory_path = Path::new(output_dir).join("memory.bin");
//     let memory_file = std::fs::File::create(&memory_path)?;
//     let mut memory_writer =
//         FileWriter::new(io::BufWriter::with_capacity(50 * 1024 * 1024, memory_file));
//     write_encoded_memory(&cairo_runner.relocated_memory, &mut memory_writer)?;
//     memory_writer.flush()?;

//     let trace_path = Path::new(output_dir).join("trace.bin");
//     let relocated_trace = cairo_runner
//         .relocated_trace
//         .as_ref()
//         .ok_or(Error::Trace(TraceError::TraceNotRelocated))?;
//     let trace_file = std::fs::File::create(&trace_path)?;
//     let mut trace_writer =
//         FileWriter::new(io::BufWriter::with_capacity(3 * 1024 * 1024, trace_file));
//     write_encoded_trace(relocated_trace, &mut trace_writer)?;
//     trace_writer.flush()?;

//     // 1. Generate air_public_inputs.json
//     let public_input = cairo_runner.get_air_public_input();
//     let public_input_json = serde_json::to_string_pretty(&public_input.unwrap()).unwrap();
//     std::fs::write(
//         Path::new(output_dir).join("air_public_inputs.json"),
//         public_input_json,
//     )?;

//     // 2. Generate air_private_inputs.json (after binary files are created)
//     let private_input = cairo_runner.get_air_private_input();
//     let private_input_serializable =
//         private_input.to_serializable("trace.bin".to_string(), "memory.bin".to_string());
//     let private_input_json = serde_json::to_string_pretty(&private_input_serializable).unwrap();
//     std::fs::write(
//         Path::new(output_dir).join("air_private_inputs.json"),
//         private_input_json,
//     )?;

//     Ok(())
// }

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input_path).unwrap();
    let input: StoneCircuitLayoutCairo = serde_json::from_str(&input_str).unwrap();

    let output_dir: &'static str = "output/";
    let program_path = "cairo/build/bankai_stone.json";
    let pie = run(program_path, input).unwrap();

    pie.write_zip_file(&Path::new(output_dir).join("pie.zip"), true)
        .unwrap();
}
