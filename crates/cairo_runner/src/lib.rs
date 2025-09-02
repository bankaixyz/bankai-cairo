#![allow(clippy::result_large_err)]
pub mod error;
pub mod hint_processor;

use crate::{error::Error, hint_processor::CustomHintProcessor};
use bankai_hints::types::StoneCircuitLayoutCairo;
use cairo_vm_base::vm::cairo_vm::{
    cairo_run::{self, cairo_run_program_with_initial_scope},
    types::{exec_scope::ExecutionScopes, layout_name::LayoutName, program::Program},
    vm::runners::cairo_pie::CairoPie,
};

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

pub fn run(
    path: &str,
    input: StoneCircuitLayoutCairo,
    log_level: &'static str,
) -> Result<CairoPie, Error> {
    let program = load_program(path)?;
    let cairo_run_config = cairo_run::CairoRunConfig {
        allow_missing_builtins: Some(true),
        layout: LayoutName::all_cairo,
        ..Default::default()
    };
    let mut hint_processor = CustomHintProcessor::new();
    let mut exec_scopes = ExecutionScopes::new();
    exec_scopes.insert_value("input", input.input);
    exec_scopes.insert_value("output", input.output);
    exec_scopes.insert_value("program_object", program.clone());
    exec_scopes.insert_value("LOG_LEVEL_CAIRO", log_level);

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
